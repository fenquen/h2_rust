use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use crate::h2_rust_common::h2_rust_constant::{NEGATIVE, POSITIVE};
use anyhow::Result;
use crate::api::error_code;
use crate::h2_rust_common::Nullable::{NotNull, Null};
use crate::message::db_error::DbError;

pub mod macros;
pub mod h2_rust_utils;
pub mod h2_rust_constant;
pub mod file_lock;

pub type Properties = HashMap<String, String>;
pub type Integer = i32;
pub type Long = i64;
pub type Byte = i8;

pub type VecRef<T> = Option<Arc<Vec<T>>>;

pub fn throw<T, E: Error + Send + Sync + 'static>(e: E) -> Result<T> {
    core::result::Result::Err(e)?
}

pub enum Nullable<T> {
    NotNull(T),
    Null,
}

impl<T> Nullable<T> {
    pub fn unwrap(&self) -> &T {
        match self {
            NotNull(t) => t,
            Null => panic!("null")
        }
    }

    pub fn unwrap_mut(&mut self) -> &mut T {
        match self {
            NotNull(t) => t,
            Null => panic!("null")
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Null => true,
            _ => false
        }
    }

    pub fn is_not_null(&self) -> bool {
        !self.is_null()
    }
}

impl<T> Default for Nullable<T> {
    fn default() -> Self {
        Null
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        if let Some(t) = value {
            NotNull(t)
        } else {
            Null
        }
    }
}

impl<T: Clone> Clone for Nullable<T> {
    fn clone(&self) -> Self {
        match self {
            NotNull(t) => NotNull(t.clone()),
            Null => Null
        }
    }
}


#[derive(Default)]
pub struct MyMutex<T> {
    mutex: Mutex<T>,
    owner_thread_id: AtomicU64,
}

pub struct MyMutexGuard<'a, T: ?Sized + 'a> {
    mutex_guard: MutexGuard<'a, T>,
    owner_thread_id: &'a AtomicU64,
}

impl<T> MyMutex<T> {
    pub fn lock(&self) -> MyMutexGuard<T> {
        let mutex_guard = self.mutex.lock().unwrap();
        self.owner_thread_id.store(thread::current().id().as_u64().get(), Ordering::Release);
        MyMutexGuard {
            mutex_guard,
            owner_thread_id: &self.owner_thread_id,
        }
    }

    pub fn is_held_by_current_thread(&self) -> bool {
        let current_thread_id = self.owner_thread_id.load(Ordering::Acquire);
        thread::current().id().as_u64().get() == current_thread_id
    }
}

impl<'a, T: ?Sized + 'a> Drop for MyMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.owner_thread_id.store(0, Ordering::Release);
    }
}