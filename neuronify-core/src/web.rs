//! Loading a file named in the page URL, as in `index.html?file=examples/leaky.nfy`.
//!
//! The fetch is asynchronous and visula's event loop never returns on the web,
//! so the result is left here for `Neuronify::update` to pick up.

use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;

thread_local! {
    static FETCHED: RefCell<Option<Result<Vec<u8>, String>>> = const { RefCell::new(None) };
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
        FETCHED.with(|fetched| *fetched.borrow_mut() = Some(result));
    });
    true
}

/// Returns the fetched file once, after the fetch has finished.
pub fn take_fetched_file() -> Option<Result<Vec<u8>, String>> {
    FETCHED.with(|fetched| fetched.borrow_mut().take())
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
