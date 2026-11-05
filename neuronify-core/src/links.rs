use egui::Response;

#[cfg(not(target_arch = "wasm32"))]
pub fn open_on_click(response: &Response, url: &str) {
    if response.clicked() {
        response.ctx.open_url(egui::OpenUrl::new_tab(url));
    }
}

#[cfg(target_arch = "wasm32")]
pub fn open_on_click(response: &Response, url: &str) {
    crate::web::open_on_pointer_up(response, url);
}
