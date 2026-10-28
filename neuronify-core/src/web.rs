//! Files in the browser: loading one named in the page URL, as in
//! `index.html?file=examples/leaky.nfy`, loading one the user picks,
//! and downloading or sharing a save.
//!
//! Fetching and reading are asynchronous and visula's event loop never returns on the web,
//! so the result is left here for `Neuronify::update` to pick up.

use std::cell::RefCell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Document, DomException, File, FilePropertyBag, HtmlAnchorElement, HtmlInputElement, Response,
    ShareData, Url,
};

const REVOKE_DELAY_MS: i32 = 10_000;
const SHARED_TYPE: &str = "application/octet-stream";

thread_local! {
    static LOADED: RefCell<Option<Result<Vec<u8>, String>>> = const { RefCell::new(None) };
}

/// Starts fetching the file named by the `file` query parameter.
/// Returns false if the page names no file.
pub fn fetch_requested_file() -> bool {
    let Some(url) = requested_file() else {
        return false;
    };
    wasm_bindgen_futures::spawn_local(async move {
        let result = fetch_bytes(&url).await.map_err(|e| {
            // fetch rejects with a TypeError, while fetch_bytes rejects with a string
            let reason = e
                .dyn_ref::<js_sys::Error>()
                .map(|error| String::from(error.message()))
                .or_else(|| e.as_string())
                .unwrap_or_else(|| format!("{e:?}"));
            format!("Could not fetch {url}: {reason}")
        });
        LOADED.with(|loaded| *loaded.borrow_mut() = Some(result));
    });
    true
}

/// Returns a fetched or picked file once, after it has been read.
pub fn take_loaded_file() -> Option<Result<Vec<u8>, String>> {
    LOADED.with(|loaded| loaded.borrow_mut().take())
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
                .map(|buffer| js_sys::Uint8Array::new(&buffer).to_vec())
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
    document.body().ok_or("no body")?.append_child(&link)?;
    link.click();
    link.remove();
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

fn requested_file() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()?
        .get("file")
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
