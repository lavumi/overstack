use crate::event::Event;

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

/// Writes an event as a JSON line and mirrors it to console.
pub fn push_event(logs: &mut Vec<String>, event: Event) {
    let line = event.to_json_line();
    log_line(&line);
    logs.push(line);
}
