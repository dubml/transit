use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::Instant,
};

use tracing::info;

#[derive(Clone, Default)]
pub struct Ready(Arc<Mutex<BTreeSet<&'static str>>>);

impl Ready {
    pub fn register_task(&self, name: &'static str) -> Task {
        self.0.lock().expect("readiness lock poisoned").insert(name);
        Task {
            ready: self.clone(),
            name,
            started: Instant::now(),
        }
    }

    pub fn pending(&self) -> Vec<&'static str> {
        self.0
            .lock()
            .expect("readiness lock poisoned")
            .iter()
            .copied()
            .collect()
    }
}

pub struct Task {
    ready: Ready,
    name: &'static str,
    started: Instant,
}

impl Drop for Task {
    fn drop(&mut self) {
        let mut pending = self.ready.0.lock().expect("readiness lock poisoned");
        pending.remove(self.name);
        if pending.is_empty() {
            info!(
                task = self.name,
                elapsed = ?self.started.elapsed(),
                "Server ready"
            );
        } else {
            info!(
                task = self.name,
                elapsed = ?self.started.elapsed(),
                pending_tasks = pending.len(),
                "Startup task completed"
            );
        }
    }
}
