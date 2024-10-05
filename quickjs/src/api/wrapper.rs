// use a sync mutex to unlnock when awaiting
use std::sync::{Arc, Mutex};
use flutter_rust_bridge::frb;
#[frb(external)]
#[frb(opaque)]
pub struct Wrapper {
    inner: Arc<Mutex<(bool, *mut ())>>,
}

impl Wrapper {
    pub fn new<T>(value: T) -> Self {
        let boxed = Box::new(value);
        let raw = Box::into_raw(boxed) as *mut ();
        Self {
            inner: Arc::new(Mutex::new((false, raw))),
        }
    }

    pub fn unsafe_ref<T>(&self) -> Option<&T> {
        let lock = self.inner.lock().unwrap();
        if lock.0 {
            None
        } else {
            unsafe { Some(&*(lock.1 as *const T)) }
        }
    }

    pub fn as_ref<T>(&self) -> Option<&T> {
        let lock = self.inner.lock().unwrap();
        if lock.0 {
            None
        } else {
            unsafe { Some(&*(lock.1 as *const T)) }
        }
    }

    pub fn as_mut<T>(&self) -> Option<&mut T> {
        let lock = self.inner.lock().unwrap();
        if lock.0 {
            None
        } else {
            unsafe { Some(&mut *(lock.1 as *mut T)) }
        }
    }
}

impl Drop for Wrapper {
    fn drop(&mut self) {
        let mut lock = self.inner.lock().unwrap();
        if !lock.0 {
            unsafe {
                drop(Box::from_raw(lock.1));
            }
            lock.0 = true;
        }
    }
}


// Ensure that Wrapper implements Sync
unsafe impl Send for Wrapper {}
unsafe impl Sync for Wrapper {}