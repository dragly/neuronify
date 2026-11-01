use std::cell::{Cell, RefCell};
use std::future::Future;
use std::rc::Rc;

use chrono::{DateTime, Utc};
use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    IdbDatabase, IdbObjectStore, IdbObjectStoreParameters, IdbRequest, IdbTransaction,
    IdbTransactionMode, Storage,
};

use super::SavedSimulation;

const DATABASE: &str = "neuronify";
const DATABASE_VERSION: u32 = 1;
const SIMULATIONS: &str = "simulations";
const COUNT_FOR_LOADING_SCREEN: &str = "neuronify-saved-simulations";
const BACKUP: &str = "neuronify-unsaved-";

type Loaded = Rc<RefCell<Option<Vec<SavedSimulation>>>>;

pub struct Store {
    database: Rc<RefCell<Option<IdbDatabase>>>,
    loaded: Loaded,
    asked_to_persist: Cell<bool>,
}

impl Store {
    pub fn open() -> Store {
        let store = Store {
            database: Rc::default(),
            loaded: Rc::default(),
            asked_to_persist: Cell::new(false),
        };
        let database = store.database.clone();
        let loaded = store.loaded.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let opened = async {
                let opened = open_database().await?;
                let simulations = read_all(&opened).await?;
                Ok::<_, JsValue>((opened, simulations))
            };
            match opened.await {
                Ok((opened, simulations)) => {
                    *database.borrow_mut() = Some(opened);
                    *loaded.borrow_mut() = Some(simulations);
                }
                Err(error) => log::error!("Could not open saved simulations: {error:?}"),
            }
        });
        store
    }

    pub fn take_loaded(&mut self) -> Option<Vec<SavedSimulation>> {
        self.loaded.borrow_mut().take()
    }

    pub fn write(&self, simulation: &SavedSimulation) {
        self.ask_to_persist();
        let record = record(simulation);
        let (id, modified) = (simulation.id.clone(), simulation.modified);
        self.change(
            move |simulations| simulations.put(&record),
            move || forget_backup(&id, modified),
        );
    }

    pub fn rename(&self, _old_name: &str, simulation: &SavedSimulation) {
        self.write(simulation);
    }

    pub fn delete(&self, simulation: &SavedSimulation) {
        forget_backup(&simulation.id, DateTime::<Utc>::MAX_UTC);
        let id = JsValue::from_str(&simulation.id);
        self.change(move |simulations| simulations.delete(&id), || {});
    }

    pub fn back_up(&self, simulation: &SavedSimulation) {
        if let (Some(storage), Some(text)) = (local_storage(), backup(simulation)) {
            let _ = storage.set_item(&backup_key(&simulation.id), &text);
        }
    }

    pub fn backups(&self) -> Vec<SavedSimulation> {
        let Some(storage) = local_storage() else {
            return Vec::new();
        };
        (0..storage.length().unwrap_or(0))
            .filter_map(|index| storage.key(index).ok().flatten())
            .filter(|key| key.starts_with(BACKUP))
            .filter_map(|key| storage.get_item(&key).ok().flatten())
            .filter_map(|text| {
                from_backup(&text)
                    .map_err(|error| log::error!("Skipped an unsaved simulation: {error:?}"))
                    .ok()
            })
            .collect()
    }

    fn change(
        &self,
        request: impl FnOnce(&IdbObjectStore) -> Result<IdbRequest, JsValue> + 'static,
        written: impl FnOnce() + 'static,
    ) {
        let opened = self.database.borrow().clone();
        match opened {
            Some(opened) => report(std::future::ready(started_in(&opened, request)), written),
            None => {
                let database = self.database.clone();
                let started = async move {
                    let opened = open_database().await?;
                    *database.borrow_mut() = Some(opened.clone());
                    started_in(&opened, request)
                };
                report(started, written);
            }
        }
    }

    fn ask_to_persist(&self) {
        if self.asked_to_persist.replace(true) {
            return;
        }
        let Some(navigator) = web_sys::window().map(|window| window.navigator()) else {
            return;
        };
        if !Reflect::has(&navigator, &"storage".into()).unwrap_or(false) {
            return;
        }
        if let Err(error) = navigator.storage().persist() {
            log::warn!("Could not ask for persistent storage: {error:?}");
        }
    }
}

pub fn remember_count(count: usize) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(COUNT_FOR_LOADING_SCREEN, &count.to_string());
    }
}

fn local_storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn backup_key(id: &str) -> String {
    format!("{BACKUP}{id}")
}

fn forget_backup(id: &str, saved: DateTime<Utc>) {
    let Some(storage) = local_storage() else {
        return;
    };
    let key = backup_key(id);
    let Ok(Some(text)) = storage.get_item(&key) else {
        return;
    };
    if !from_backup(&text).is_ok_and(|backup| backup.modified > saved) {
        let _ = storage.remove_item(&key);
    }
}

fn started_in(
    database: &IdbDatabase,
    request: impl FnOnce(&IdbObjectStore) -> Result<IdbRequest, JsValue>,
) -> Result<IdbTransaction, JsValue> {
    let transaction =
        database.transaction_with_str_and_mode(SIMULATIONS, IdbTransactionMode::Readwrite)?;
    request(&transaction.object_store(SIMULATIONS)?)?;
    Ok(transaction)
}

fn report(
    started: impl Future<Output = Result<IdbTransaction, JsValue>> + 'static,
    written: impl FnOnce() + 'static,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let changed = async { committed(&started.await?).await };
        match changed.await {
            Ok(_) => written(),
            Err(error) => log::error!("Could not save: {error:?}"),
        }
    });
}

fn committed(transaction: &IdbTransaction) -> JsFuture {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let on_complete = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::UNDEFINED);
        });
        let aborted = transaction.clone();
        let on_abort = Closure::once_into_js(move || {
            let error = aborted.error().map(JsValue::from);
            let _ = reject.call1(&JsValue::UNDEFINED, &error.unwrap_or_default());
        });
        transaction.set_oncomplete(Some(on_complete.unchecked_ref()));
        transaction.set_onabort(Some(on_abort.unchecked_ref()));
    });
    JsFuture::from(promise)
}

fn finished(request: &IdbRequest) -> JsFuture {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let succeeded = request.clone();
        let on_success = Closure::once_into_js(move || {
            let _ = resolve.call1(&JsValue::UNDEFINED, &succeeded.result().unwrap_or_default());
        });
        let failed = request.clone();
        let on_error = Closure::once_into_js(move || {
            let error = failed.error().ok().flatten().map(JsValue::from);
            let _ = reject.call1(&JsValue::UNDEFINED, &error.unwrap_or_default());
        });
        request.set_onsuccess(Some(on_success.unchecked_ref()));
        request.set_onerror(Some(on_error.unchecked_ref()));
    });
    JsFuture::from(promise)
}

async fn open_database() -> Result<IdbDatabase, JsValue> {
    let request = web_sys::window()
        .ok_or("no global window")?
        .indexed_db()?
        .ok_or("this browser has no IndexedDB")?
        .open_with_u32(DATABASE, DATABASE_VERSION)?;
    let upgraded = request.clone();
    let create_store = Closure::once_into_js(move || {
        let Ok(database) = upgraded.result() else {
            return;
        };
        let parameters = IdbObjectStoreParameters::new();
        parameters.set_key_path(&"id".into());
        let database: IdbDatabase = database.unchecked_into();
        if let Err(error) =
            database.create_object_store_with_optional_parameters(SIMULATIONS, &parameters)
        {
            log::error!("Could not create the store for saved simulations: {error:?}");
        }
    });
    request.set_onupgradeneeded(Some(create_store.unchecked_ref()));
    Ok(finished(&request).await?.unchecked_into())
}

async fn read_all(database: &IdbDatabase) -> Result<Vec<SavedSimulation>, JsValue> {
    let request = database
        .transaction_with_str_and_mode(SIMULATIONS, IdbTransactionMode::Readonly)?
        .object_store(SIMULATIONS)?
        .get_all()?;
    let records: js_sys::Array = finished(&request).await?.dyn_into()?;
    Ok(records
        .iter()
        .filter_map(|record| {
            simulation(&record)
                .map_err(|error| log::error!("Skipped a saved simulation: {error:?}"))
                .ok()
        })
        .collect())
}

fn record(simulation: &SavedSimulation) -> Object {
    let record = Object::new();
    let fields: [(&str, JsValue); 4] = [
        ("id", simulation.id.as_str().into()),
        ("name", simulation.name.as_str().into()),
        (
            "modified",
            (simulation.modified.timestamp_millis() as f64).into(),
        ),
        ("bytes", Uint8Array::from(&simulation.bytes[..]).into()),
    ];
    for (key, value) in fields {
        Reflect::set(&record, &key.into(), &value).unwrap();
    }
    record
}

fn backup(simulation: &SavedSimulation) -> Option<String> {
    let binary: String = simulation
        .bytes
        .iter()
        .map(|&byte| char::from(byte))
        .collect();
    let record = record(simulation);
    let encoded = web_sys::window()?.btoa(&binary).ok()?;
    Reflect::set(&record, &"bytes".into(), &encoded.into()).ok()?;
    js_sys::JSON::stringify(&record).ok().map(String::from)
}

fn from_backup(text: &str) -> Result<SavedSimulation, JsValue> {
    let record = js_sys::JSON::parse(text)?;
    let encoded = Reflect::get(&record, &"bytes".into())?
        .as_string()
        .ok_or("bytes is not text")?;
    let bytes = web_sys::window()
        .ok_or("no global window")?
        .atob(&encoded)?
        .chars()
        .map(u8::try_from)
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "bytes is not binary")?;
    Reflect::set(&record, &"bytes".into(), &Uint8Array::from(&bytes[..]))?;
    simulation(&record)
}

fn simulation(record: &JsValue) -> Result<SavedSimulation, JsValue> {
    let field = |key: &str| Reflect::get(record, &key.into());
    let text = |key: &str| -> Result<String, JsValue> {
        field(key)?
            .as_string()
            .ok_or_else(|| format!("{key} is not text").into())
    };
    let milliseconds = field("modified")?
        .as_f64()
        .ok_or("modified is not a number")?;
    Ok(SavedSimulation {
        id: text("id")?,
        name: text("name")?,
        modified: DateTime::from_timestamp_millis(milliseconds as i64).unwrap_or_default(),
        bytes: field("bytes")?.dyn_into::<Uint8Array>()?.to_vec(),
    })
}
