#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
mod folder;

use chrono::{DateTime, Utc};

#[cfg(target_arch = "wasm32")]
use browser::Store;
#[cfg(not(target_arch = "wasm32"))]
use folder::Store;

pub const UNTITLED: &str = "Untitled";
const AUTOSAVE_DELAY_SECONDS: f64 = 1.5;

#[derive(Clone, Debug, PartialEq)]
pub struct SavedSimulation {
    pub id: String,
    pub name: String,
    pub modified: DateTime<Utc>,
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct Library {
    simulations: Vec<SavedSimulation>,
    store: Option<Store>,
    revision: u64,
}

impl Library {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open() -> Library {
        let Some(store) = Store::in_data_directory() else {
            log::error!("Found no folder to save simulations in");
            return Library::default();
        };
        Library::in_folder(store)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn in_folder(store: Store) -> Library {
        let mut library = Library {
            simulations: store.read_all(),
            store: Some(store),
            revision: 0,
        };
        library.sort();
        library
    }

    #[cfg(target_arch = "wasm32")]
    pub fn open() -> Library {
        Library {
            store: Some(Store::open()),
            ..Default::default()
        }
    }

    pub fn poll(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(loaded) = self.store.as_mut().and_then(Store::take_loaded) {
            for simulation in loaded {
                if self.get(&simulation.id).is_none() {
                    self.simulations.push(simulation);
                }
            }
            self.changed();
            let unsaved = self.store.as_ref().map(Store::backups).unwrap_or_default();
            for simulation in unsaved {
                self.save(simulation);
            }
        }
    }

    pub fn simulations(&self) -> &[SavedSimulation] {
        &self.simulations
    }

    pub fn get(&self, id: &str) -> Option<&SavedSimulation> {
        self.simulations
            .iter()
            .find(|simulation| simulation.id == id)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn create(&mut self, name: &str, bytes: Vec<u8>) -> String {
        let id = new_id();
        self.save(SavedSimulation {
            id: id.clone(),
            name: name.to_owned(),
            modified: Utc::now(),
            bytes,
        });
        id
    }

    pub fn save(&mut self, simulation: SavedSimulation) {
        let index = match self.simulations.iter().position(|s| s.id == simulation.id) {
            Some(index) => {
                let saved = &mut self.simulations[index];
                saved.bytes = simulation.bytes;
                saved.modified = simulation.modified;
                index
            }
            None => {
                let name = self.unique_name(&simulation.name, None);
                self.simulations
                    .push(SavedSimulation { name, ..simulation });
                self.simulations.len() - 1
            }
        };
        if let Some(store) = &self.store {
            store.write(&self.simulations[index]);
        }
        self.changed();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn back_up(&self, simulation: &SavedSimulation) {
        if let Some(store) = &self.store {
            store.back_up(simulation);
        }
    }

    pub fn rename(&mut self, id: &str, name: &str) {
        if name.trim().is_empty() {
            return;
        }
        let name = self.unique_name(name, Some(id));
        let Some(simulation) = self.simulations.iter_mut().find(|s| s.id == id) else {
            return;
        };
        let old_name = std::mem::replace(&mut simulation.name, name);
        if let Some(store) = &self.store {
            store.rename(&old_name, simulation);
        }
        self.changed();
    }

    pub fn delete(&mut self, id: &str) {
        let Some(index) = self.simulations.iter().position(|s| s.id == id) else {
            return;
        };
        let simulation = self.simulations.remove(index);
        if let Some(store) = &self.store {
            store.delete(&simulation);
        }
        self.changed();
    }

    fn changed(&mut self) {
        self.sort();
        self.revision += 1;
        #[cfg(target_arch = "wasm32")]
        browser::remember_count(self.simulations.len());
    }

    fn sort(&mut self) {
        self.simulations
            .sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)));
    }

    fn unique_name(&self, wanted: &str, renamed: Option<&str>) -> String {
        let wanted = match wanted.trim() {
            "" => UNTITLED,
            trimmed => trimmed,
        };
        let taken = |name: &str| {
            self.simulations.iter().any(|simulation| {
                Some(simulation.id.as_str()) != renamed
                    && simulation.name.to_lowercase() == name.to_lowercase()
            })
        };
        (1..)
            .map(|number| match number {
                1 => wanted.to_owned(),
                _ => format!("{wanted} {number}"),
            })
            .find(|name| !taken(name))
            .unwrap()
    }
}

fn new_id() -> String {
    format!("{:016x}", rand::random::<u64>())
}

pub struct Autosave {
    id: String,
    name: String,
    last_edit: Option<f64>,
}

impl Autosave {
    pub fn copy_of(name: &str) -> Autosave {
        Autosave {
            id: new_id(),
            name: name.to_owned(),
            last_edit: None,
        }
    }

    pub fn saved(library: &Library, id: String) -> Autosave {
        Autosave {
            name: library
                .get(&id)
                .map_or(UNTITLED.to_owned(), |simulation| simulation.name.clone()),
            id,
            last_edit: None,
        }
    }

    pub fn edited(&mut self, now: f64) {
        self.last_edit = Some(now);
    }

    pub fn has_unsaved_edits(&self) -> bool {
        self.last_edit.is_some()
    }

    pub fn is_due(&self, now: f64) -> bool {
        self.last_edit
            .is_some_and(|edited| now - edited >= AUTOSAVE_DELAY_SECONDS)
    }

    pub fn save(&mut self, library: &mut Library, bytes: Vec<u8>) {
        self.last_edit = None;
        library.save(self.simulation(bytes));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn back_up(&self, library: &Library, bytes: Vec<u8>) {
        library.back_up(&self.simulation(bytes));
    }

    fn simulation(&self, bytes: Vec<u8>) -> SavedSimulation {
        SavedSimulation {
            id: self.id.clone(),
            name: self.name.clone(),
            modified: Utc::now(),
            bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_stay_unique_regardless_of_case() {
        let mut library = Library::default();
        let first = library.create("Leaky", vec![]);
        let second = library.create("leaky", vec![]);
        let third = library.create("Leaky", vec![]);
        let names: Vec<&str> = [&first, &second, &third]
            .map(|id| library.get(id).unwrap().name.as_str())
            .to_vec();
        assert_eq!(names, ["Leaky", "leaky 2", "Leaky 3"]);
    }

    #[test]
    fn renaming_keeps_names_unique_and_ignores_blank_names() {
        let mut library = Library::default();
        library.create("Leaky", vec![]);
        let other = library.create("Circuit", vec![]);
        library.rename(&other, "Leaky");
        assert_eq!(library.get(&other).unwrap().name, "Leaky 2");
        library.rename(&other, "  ");
        assert_eq!(library.get(&other).unwrap().name, "Leaky 2");
        library.rename(&other, "Leaky 2");
        assert_eq!(library.get(&other).unwrap().name, "Leaky 2");
    }

    #[test]
    fn newest_first() {
        let mut library = Library::default();
        let older = library.create("Older", vec![]);
        let newer = library.create("Newer", vec![]);
        for simulation in &mut library.simulations {
            simulation.modified = DateTime::UNIX_EPOCH;
            if simulation.id == newer {
                simulation.modified += chrono::Duration::days(1);
            }
        }
        library.sort();
        let order = |library: &Library| -> Vec<String> {
            library.simulations().iter().map(|s| s.id.clone()).collect()
        };
        assert_eq!(order(&library), [newer.clone(), older.clone()]);
        Autosave::saved(&library, older.clone()).save(&mut library, vec![1]);
        assert_eq!(order(&library), [older, newer]);
    }

    #[test]
    fn every_change_bumps_the_revision() {
        let mut library = Library::default();
        let id = library.create("Leaky", vec![]);
        Autosave::saved(&library, id.clone()).save(&mut library, vec![1]);
        library.rename(&id, "Renamed");
        library.delete(&id);
        assert_eq!(library.revision(), 4);
        assert!(library.simulations().is_empty());
    }

    #[test]
    fn saving_updates_a_known_simulation_or_adds_it_under_a_unique_name() {
        let mut library = Library::default();
        let id = library.create("Leaky", vec![1]);
        let simulation = |id: &str, bytes: Vec<u8>| SavedSimulation {
            id: id.to_owned(),
            name: "Leaky".to_owned(),
            modified: Utc::now(),
            bytes,
        };
        library.save(simulation(&id, vec![2]));
        library.save(simulation("unsaved", vec![3]));
        let saved = |id: &str| {
            let simulation = library.get(id).unwrap();
            (simulation.name.as_str(), &simulation.bytes[..])
        };
        assert_eq!(saved(&id), ("Leaky", &[2][..]));
        assert_eq!(saved("unsaved"), ("Leaky 2", &[3][..]));
    }

    #[test]
    fn a_copy_is_saved_on_its_first_edit_and_updated_after_that() {
        let mut library = Library::default();
        let mut autosave = Autosave::copy_of("Leaky");
        assert!(!autosave.is_due(10.0));

        autosave.edited(1.0);
        assert!(!autosave.is_due(2.0));
        assert!(autosave.is_due(2.5));
        autosave.save(&mut library, vec![1]);
        assert!(!autosave.has_unsaved_edits());
        let id = autosave.id.clone();
        assert_eq!(library.get(&id).unwrap().name, "Leaky");

        autosave.edited(3.0);
        autosave.save(&mut library, vec![2]);
        assert_eq!(library.simulations().len(), 1);
        assert_eq!(library.get(&id).unwrap().bytes, [2]);
    }

    #[test]
    fn a_deleted_simulation_comes_back_when_edited_again() {
        let mut library = Library::default();
        let id = library.create("Leaky", vec![1]);
        let mut autosave = Autosave::saved(&library, id.clone());
        library.delete(&id);

        autosave.edited(0.0);
        autosave.save(&mut library, vec![2]);
        let restored = library.get(&id).unwrap();
        assert_eq!(
            (restored.name.as_str(), &restored.bytes[..]),
            ("Leaky", &[2][..])
        );
    }
}
