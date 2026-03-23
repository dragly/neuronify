#[cfg(target_arch = "wasm32")]
use js_sys::Uint8Array;
#[cfg(target_arch = "wasm32")]
use std::borrow::BorrowMut;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use visula::winit::event::{Event, WindowEvent};
#[cfg(target_arch = "wasm32")]
use visula::{
    create_event_loop, initialize_logger, Application, CustomEvent, RunConfig,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestInit, Response};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;
#[cfg(target_arch = "wasm32")]
use visula::winit::platform::web::EventLoopExtWebSys;

pub mod app;
pub mod components;
pub mod constants;
pub mod input;
pub mod legacy;
pub mod measurement;
pub mod rendering;
pub mod serialization;
pub mod simulation;
pub mod tools;

pub use app::{Error, Neuronify};
pub use components::*;
pub use constants::*;
pub use input::{Keyboard, Mouse};
pub use simulation::{fhn_step, lif_step, run_headless, SpikeRecord};
pub use tools::*;

#[cfg(target_arch = "wasm32")]
struct Bundle {
    application: Application,
    simulation: Neuronify,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmWrapper {
    event_loop: EventLoop<CustomEvent>,
    bundles: Vec<Bundle>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn initialize() -> WasmWrapper {
    initialize_logger();
    let event_loop = create_event_loop();
    let bundles: Vec<Bundle> = Vec::new();
    WasmWrapper {
        event_loop,
        bundles,
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn load(wrapper: &mut WasmWrapper, canvas: &str, url: &str) -> Result<(), JsValue> {
    // TODO: Rework for winit 0.30 ApplicationHandler pattern
    // create_window now requires &ActiveEventLoop which is only available inside ApplicationHandler
    let window = visula::create_window_with_config(
        &RunConfig {
            canvas_name: canvas.to_owned(),
        },
        todo!("Need ActiveEventLoop from ApplicationHandler"),
    );
    let mut application = Application::new(window).await;

    let mut opts = RequestInit::new();
    opts.method("GET");
    let request = Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().ok_or("No global `window` exists")?;
    let response_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response_value.dyn_into()?;
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = Uint8Array::new(&buffer);
    let vec = uint8_array.to_vec();
    let simulation = Neuronify::from_slice(&mut application, &vec);
    wrapper.bundles.push(Bundle {
        application,
        simulation,
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(mut wrapper: WasmWrapper) -> Result<(), JsValue> {
    // TODO: Rework for winit 0.30 ApplicationHandler pattern
    // The old closure-based event loop API no longer exists
    Ok(())
}

pub fn run() {
    visula::run(Neuronify::new);
}
