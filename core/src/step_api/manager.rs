use std::cell::RefCell;
use std::collections::HashMap;

use crate::engine::runtime::ActiveRun;
use crate::model::PlayerInitStats;

#[derive(Default)]
struct RunManager {
    next_handle: u32,
    runs: HashMap<u32, ActiveRun>,
}

impl RunManager {
    fn create_run(&mut self, seed: u32, max_nodes: u32) -> u32 {
        self.next_handle = self.next_handle.saturating_add(1).max(1);
        let handle = self.next_handle;
        self.runs
            .insert(handle, ActiveRun::new(seed as u64, max_nodes));
        handle
    }

    fn create_run_with_stats(&mut self, seed: u32, max_nodes: u32, stats: PlayerInitStats) -> u32 {
        self.next_handle = self.next_handle.saturating_add(1).max(1);
        let handle = self.next_handle;
        self.runs.insert(
            handle,
            ActiveRun::new_with_stats(seed as u64, max_nodes, Some(stats)),
        );
        handle
    }

    fn destroy_run(&mut self, handle: u32) {
        self.runs.remove(&handle);
    }

    fn reset_run(&mut self, handle: u32) -> bool {
        if let Some(run) = self.runs.get_mut(&handle) {
            run.reset();
            true
        } else {
            false
        }
    }
}

thread_local! {
    static MANAGER: RefCell<RunManager> = RefCell::new(RunManager::default());
}

pub(super) fn create_run(seed: u32, max_nodes: u32) -> u32 {
    MANAGER.with(|manager| manager.borrow_mut().create_run(seed, max_nodes))
}

pub(super) fn create_run_with_stats(seed: u32, max_nodes: u32, stats: PlayerInitStats) -> u32 {
    MANAGER.with(|manager| {
        manager
            .borrow_mut()
            .create_run_with_stats(seed, max_nodes, stats)
    })
}

pub(super) fn destroy_run(handle: u32) {
    MANAGER.with(|manager| manager.borrow_mut().destroy_run(handle));
}

pub(super) fn reset_run(handle: u32) -> bool {
    MANAGER.with(|manager| manager.borrow_mut().reset_run(handle))
}

pub(super) fn with_run_mut<T>(handle: u32, f: impl FnOnce(&mut ActiveRun) -> T) -> Option<T> {
    MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.runs.get_mut(&handle).map(f)
    })
}

pub(super) fn with_run<T>(handle: u32, f: impl FnOnce(&ActiveRun) -> T) -> Option<T> {
    MANAGER.with(|manager| {
        let manager = manager.borrow();
        manager.runs.get(&handle).map(f)
    })
}
