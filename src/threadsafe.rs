use std::sync::{Arc, Mutex};

///Helper struct for wrapping Arc<Mutex<T>> to help with readibility
pub struct ThreadSafe<T> {
    obj: Arc<Mutex<T>>
}

impl <T>ThreadSafe<T> {
    pub fn lock(&self)->Result<std::sync::MutexGuard<'_, T>,
    std::sync::PoisonError<std::sync::MutexGuard<'_, T>>>
    {
        return self.obj.lock()
    }

    pub fn clone(&self)->ThreadSafe<T> {
        return ThreadSafe{obj: self.obj.clone()}
    }

    pub fn from(obj: T)->ThreadSafe<T> {
        return ThreadSafe{obj: Arc::from(Mutex::from(obj))}
    }
}