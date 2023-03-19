use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub struct Shared<T>(Arc<Mutex<T>>);

unsafe impl<T> Send for Shared<T> {}
unsafe impl<T> Sync for Shared<T> {}

impl<T> Shared<T> {
    pub fn new(data: T) -> Self {
        Self(Arc::new(Mutex::new(data)))
    }

    pub async fn lock(&self) -> MutexGuard<T> {
        self.0.lock().await
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}