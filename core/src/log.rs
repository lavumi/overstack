use crate::event::Event;
use std::cell::Cell;

thread_local! {
    static CURRENT_LOG_TICK: Cell<u32> = const { Cell::new(0) };
}

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
    let mut line = event.to_json_line();
    let tick = CURRENT_LOG_TICK.with(|v| v.get());
    if line.starts_with('{') {
        line.insert_str(1, &format!("\"tick\":{tick},"));
    }
    log_line(&line);
    logs.push(line);
}

pub fn set_log_tick(tick: u32) {
    CURRENT_LOG_TICK.with(|v| v.set(tick));
}
