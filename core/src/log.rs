#[cfg(target_arch = "wasm32")]
mod wasm_log {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(message: &str);
    }

    pub fn log_line(message: &str) {
        log(message);
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_log::log_line;

#[cfg(not(target_arch = "wasm32"))]
pub fn log_line(message: &str) {
    println!("{message}");
}
