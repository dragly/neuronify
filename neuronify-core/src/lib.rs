use js_sys::Uint8Array;
use std::borrow::BorrowMut;
use std::sync::Arc;
use visula::winit::event::{Event, WindowEvent};
use visula::{
    create_event_loop, create_window, initialize_logger, Application, CustomEvent, RunConfig,
    Simulation,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopWindowTarget;
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

struct Bundle {
    application: Application,
    simulation: Neuronify,
}

#[wasm_bindgen]
pub struct WasmWrapper {
    event_loop: EventLoop<CustomEvent>,
    bundles: Vec<Bundle>,
}

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

#[wasm_bindgen]
pub async fn load(wrapper: &mut WasmWrapper, canvas: &str, url: &str) -> Result<(), JsValue> {
    let window = create_window(
        RunConfig {
            canvas_name: canvas.to_owned(),
        },
        &wrapper.event_loop,
    );
    let mut application = pollster::block_on(async { Application::new(Arc::new(window)).await });

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

#[wasm_bindgen]
pub async fn start(mut wrapper: WasmWrapper) -> Result<(), JsValue> {
    let _event_handler = move |event, target: &EventLoopWindowTarget<CustomEvent>| {
        for bundle in wrapper.bundles.iter_mut() {
            let application = &mut bundle.application;
            let simulation = &mut bundle.simulation;
            if !application.handle_event(&event) {
                simulation.handle_event(application, &event);
            }
            if let Event::WindowEvent { ref event, .. } = event {
                match event {
                    WindowEvent::RedrawRequested => {
                        application.update();
                        simulation.update(application);
                        application.render(simulation);

                        application.window.borrow_mut().request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
        }
    };
    #[cfg(target_arch = "wasm32")]
    wrapper.event_loop.spawn(_event_handler);
    Ok(())
}

pub fn run() {
    visula::run(Neuronify::new);
}
