use std::{
    panic::{self, AssertUnwindSafe},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use merc_error::Error;

use crate::tasks::{Spawn, Task};

#[derive(Clone)]
pub struct ThreadRuntime {
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl ThreadRuntime {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn join_all(&self) {
        let handles: Vec<_> = self.handles.lock().unwrap().drain(..).collect();

        for handle in handles {
            let _ = handle.join();
        }
    }

    pub fn thread_count(&self) -> usize {
        self.handles.lock().unwrap().len()
    }
}

impl Spawn for ThreadRuntime {
    fn spawn<T, H>(&self, handler: H) -> Task<T>
    where
        T: Send + 'static,
        H: FnOnce() -> T + Send + 'static,
    {
        let task = Task::<T>::new();
        let result = task.result();
        let handle = thread::spawn(move || {
            let outcome = panic::catch_unwind(AssertUnwindSafe(handler));

            match outcome {
                Ok(value) => result.ok(value),
                Err(panic_info) => result.throw(Error::panic(panic_info)),
            }
        });

        self.handles.lock().unwrap().push(handle);
        task
    }
}

impl Default for ThreadRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ThreadRuntime {
    fn drop(&mut self) {
        self.join_all();
    }
}
