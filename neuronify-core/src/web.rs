//! Files in the browser: loading one named in the page URL, as in
//! `index.html?file=examples/tutorial/2_leaks.neuronify`, loading one the user picks,
//! and downloading or sharing a save.
//! Also hides the loading screen of `index.html` once there is something to show.
//!
//! Fetching and reading are asynchronous and visula's event loop never returns on the web,
//! so the result is left here for `Neuronify::update` to pick up.

use std::cell::{Cell, RefCell};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AddEventListenerOptions, Document, DomException, File, FilePropertyBag, HtmlAnchorElement,
    HtmlElement, HtmlInputElement, Response, ShareData, Url,
};

const REVOKE_DELAY_MS: i32 = 10_000;
const TICK_MS: u32 = 10;
const SHARED_TYPE: &str = "application/octet-stream";

struct PressedLink {
    url: String,
    context: egui::Context,
    pass: u64,
}

pub struct LoadedFile {
    pub name: String,
    pub bytes: Vec<u8>,
    pub picked: bool,
    pub url: Option<String>,
}

thread_local! {
    static LOADED: RefCell<Option<Result<LoadedFile, String>>> = const { RefCell::new(None) };
    static FETCHING: Cell<bool> = const { Cell::new(false) };
    static MISSING_LINK: Cell<bool> = const { Cell::new(false) };
    static LOADING_SCREEN_SHOWN: Cell<bool> = const { Cell::new(true) };
    static PRESSED_LINK: RefCell<Option<PressedLink>> = const { RefCell::new(None) };
    static OPENED_ON_POINTER_UP: RefCell<Option<String>> = const { RefCell::new(None) };
    static LISTENING_FOR_LINK_RELEASE: Cell<bool> = const { Cell::new(false) };
}

/// Starts fetching the file named by the `file` query parameter.
/// Returns false if the page names no file.
pub fn fetch_requested_file() -> bool {
    let Some(url) = requested_file() else {
        return false;
    };
    FETCHING.set(true);
    wasm_bindgen_futures::spawn_local(async move {
        let result = fetch_bytes(&url)
            .await
            .map(|bytes| LoadedFile {
                name: name_in(&url),
                bytes,
                picked: false,
                url: absolute(&url),
            })
            .map_err(|e| {
                // fetch rejects with a TypeError, while fetch_bytes rejects with a string
                let reason = e
                    .dyn_ref::<js_sys::Error>()
                    .map(|error| String::from(error.message()))
                    .or_else(|| e.as_string())
                    .unwrap_or_else(|| format!("{e:?}"));
                format!("Could not fetch {url}: {reason}")
            });
        LOADED.with(|loaded| *loaded.borrow_mut() = Some(result));
        FETCHING.set(false);
    });
    true
}

/// Returns a fetched or picked file once, after it has been read.
pub fn take_loaded_file() -> Option<Result<LoadedFile, String>> {
    LOADED.with(|loaded| loaded.borrow_mut().take())
}

pub fn fetch_beside(origin: &str, name: &str) -> bool {
    let Some(url) = Url::new_with_base(name, origin).ok().map(|url| url.href()) else {
        return false;
    };
    wasm_bindgen_futures::spawn_local(async move {
        match fetch_bytes(&url).await {
            Ok(bytes) => {
                let file = LoadedFile {
                    name: name_in(&url),
                    bytes,
                    picked: false,
                    url: Some(url),
                };
                LOADED.with(|loaded| *loaded.borrow_mut() = Some(Ok(file)));
            }
            Err(error) => {
                log::warn!("Could not fetch {url}: {error:?}");
                MISSING_LINK.set(true);
            }
        }
    });
    true
}

pub fn take_missing_link() -> bool {
    MISSING_LINK.replace(false)
}

fn absolute(url: &str) -> Option<String> {
    let page = web_sys::window()?.location().href().ok()?;
    Url::new_with_base(url, &page).ok().map(|url| url.href())
}

pub fn hide_loading_screen_when_ready() {
    if FETCHING.get() || !LOADING_SCREEN_SHOWN.replace(false) {
        return;
    }
    if let Some(screen) = document()
        .ok()
        .and_then(|document| document.get_element_by_id("loading"))
    {
        screen.set_class_name("done");
    }
}

pub fn pick_file() -> Result<(), JsValue> {
    let document = document()?;
    let input: HtmlInputElement = document.create_element("input")?.dyn_into()?;
    input.set_type("file");
    input.set_hidden(true);
    let picked = input.clone();
    let on_change = Closure::once_into_js(move || {
        picked.remove();
        let Some(file) = picked.files().and_then(|files| files.get(0)) else {
            return;
        };
        wasm_bindgen_futures::spawn_local(async move {
            let result = JsFuture::from(file.array_buffer())
                .await
                .map(|buffer| LoadedFile {
                    name: stem(&file.name()).to_owned(),
                    bytes: js_sys::Uint8Array::new(&buffer).to_vec(),
                    picked: true,
                    url: None,
                })
                .map_err(|error| format!("Could not read {}: {error:?}", file.name()));
            LOADED.with(|loaded| *loaded.borrow_mut() = Some(result));
        });
    });
    input.set_onchange(Some(on_change.unchecked_ref()));
    document.body().ok_or("no body")?.append_child(&input)?;
    input.click();
    Ok(())
}

pub fn download(bytes: &[u8], name: &str) -> Result<(), JsValue> {
    let document = document()?;
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let url =
        Url::create_object_url_with_blob(&web_sys::Blob::new_with_u8_array_sequence(&parts)?)?;
    let link: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    link.set_href(&url);
    link.set_download(name);
    click(&document, &link)?;
    let revoke = Closure::once_into_js(move || Url::revoke_object_url(&url));
    web_sys::window()
        .ok_or("no global window")?
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            revoke.unchecked_ref(),
            REVOKE_DELAY_MS,
        )?;
    Ok(())
}

pub fn can_share(name: &str) -> bool {
    let Some(navigator) = web_sys::window().map(|window| window.navigator()) else {
        return false;
    };
    js_sys::Reflect::has(&navigator, &"canShare".into()).unwrap_or(false)
        && file(&[], name).is_ok_and(|file| navigator.can_share_with_data(&share_data(&file)))
}

pub fn is_ios() -> bool {
    let Some(navigator) = web_sys::window().map(|window| window.navigator()) else {
        return false;
    };
    let agent = navigator.user_agent().unwrap_or_default();
    let ipad_posing_as_mac = agent.contains("Macintosh") && navigator.max_touch_points() > 1;
    ["iPhone", "iPad", "iPod"]
        .iter()
        .any(|device| agent.contains(device))
        || ipad_posing_as_mac
}

pub fn share(bytes: &[u8], name: &str) -> Result<(), JsValue> {
    let navigator = web_sys::window().ok_or("no global window")?.navigator();
    let shared = navigator.share_with_data(&share_data(&file(bytes, name)?));
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = JsFuture::from(shared).await {
            let cancelled = error
                .dyn_ref::<DomException>()
                .is_some_and(|error| error.name() == "AbortError");
            if !cancelled {
                log::error!("Could not share: {error:?}");
            }
        }
    });
    Ok(())
}

pub fn open_on_pointer_up(response: &egui::Response, url: &str) {
    if response.is_pointer_button_down_on() && response.contains_pointer() {
        PRESSED_LINK.with(|pressed| {
            *pressed.borrow_mut() = Some(PressedLink {
                url: url.to_owned(),
                context: response.ctx.clone(),
                pass: response.ctx.cumulative_pass_nr(),
            })
        });
        listen_for_link_release();
    }
    if response.clicked() {
        let opened = OPENED_ON_POINTER_UP.with(|opened| opened.borrow_mut().take());
        if opened.as_deref() != Some(url) {
            open_in_new_tab(url);
        }
    }
}

fn listen_for_link_release() {
    if LISTENING_FOR_LINK_RELEASE.with(|listening| listening.replace(true)) {
        return;
    }
    let on_pointer_up = Closure::once_into_js(|| {
        LISTENING_FOR_LINK_RELEASE.with(|listening| listening.set(false));
        let pressed = PRESSED_LINK.with(|pressed| pressed.borrow_mut().take());
        let released = pressed.filter(|link| link.context.cumulative_pass_nr() == link.pass + 1);
        if let Some(link) = &released {
            open_in_new_tab(&link.url);
        }
        OPENED_ON_POINTER_UP.with(|opened| *opened.borrow_mut() = released.map(|link| link.url));
    });
    let once = AddEventListenerOptions::new();
    once.set_once(true);
    let listening = web_sys::window().map(|window| {
        window.add_event_listener_with_callback_and_add_event_listener_options(
            "pointerup",
            on_pointer_up.unchecked_ref(),
            &once,
        )
    });
    if let Some(Err(error)) = listening {
        LISTENING_FOR_LINK_RELEASE.with(|listening| listening.set(false));
        log::error!("Could not listen for the link's release: {error:?}");
    }
}

fn open_in_new_tab(url: &str) {
    if let Err(error) = click_link_to_new_tab(url) {
        log::error!("Could not open {url}: {error:?}");
    }
}

fn click_link_to_new_tab(url: &str) -> Result<(), JsValue> {
    let document = document()?;
    let link: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    link.set_href(url);
    link.set_target("_blank");
    click(&document, &link)
}

fn click(document: &Document, link: &HtmlAnchorElement) -> Result<(), JsValue> {
    document.body().ok_or("no body")?.append_child(link)?;
    link.click();
    link.remove();
    Ok(())
}

pub fn tick() {
    let Some(navigator) = web_sys::window().map(|window| window.navigator()) else {
        return;
    };
    if js_sys::Reflect::has(&navigator, &"vibrate".into()).unwrap_or(false) {
        navigator.vibrate_with_duration(TICK_MS);
    } else if is_ios() {
        if let Err(error) = flip_hidden_switch() {
            log::error!("Could not tick: {error:?}");
        }
    }
}

fn flip_hidden_switch() -> Result<(), JsValue> {
    let document = document()?;
    let label: HtmlElement = document.create_element("label")?.dyn_into()?;
    let switch: HtmlInputElement = document.create_element("input")?.dyn_into()?;
    switch.set_type("checkbox");
    switch.set_attribute("switch", "")?;
    label.set_hidden(true);
    label.append_child(&switch)?;
    document.body().ok_or("no body")?.append_child(&label)?;
    label.click();
    label.remove();
    Ok(())
}

pub fn prompt(message: &str, default: &str) -> Option<String> {
    web_sys::window()?
        .prompt_with_message_and_default(message, default)
        .ok()
        .flatten()
}

fn file(bytes: &[u8], name: &str) -> Result<File, JsValue> {
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let options = FilePropertyBag::new();
    options.set_type(SHARED_TYPE);
    File::new_with_u8_array_sequence_and_options(&parts, name, &options)
}

fn share_data(file: &File) -> ShareData {
    let data = ShareData::new();
    data.set_files(&js_sys::Array::of1(file));
    data
}

fn document() -> Result<Document, JsValue> {
    Ok(web_sys::window()
        .and_then(|window| window.document())
        .ok_or("no document")?)
}

fn stem(file_name: &str) -> &str {
    file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem)
}

fn name_in(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or_default();
    let file_name = stem(path.rsplit('/').next().unwrap_or_default());
    js_sys::decode_uri_component(file_name)
        .map(String::from)
        .unwrap_or_else(|_| file_name.to_owned())
}

fn requested_file() -> Option<String> {
    search_parameter("file")
}

pub fn debug_view() -> Option<String> {
    search_parameter("debug")
}

fn search_parameter(name: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()?
        .get(name)
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, JsValue> {
    let window = web_sys::window().ok_or("no global window")?;
    let response: Response = JsFuture::from(window.fetch_with_str(url))
        .await?
        .dyn_into()?;
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()).into());
    }
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}
