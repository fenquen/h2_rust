use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Weak;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use crate::h2_rust_common::h2_rust_constant::{NEGATIVE, POSITIVE};
use anyhow::Result;
use crate::api::error_code;
use crate::message::db_error::DbError;

pub mod h2_rust_macros;
pub mod h2_rust_utils;
pub mod h2_rust_constant;
pub mod file_lock;
pub mod h2_rust_cell;
pub mod h2_rust_type;
pub mod byte_buffer;

pub type Properties = HashMap<String, String>;
pub type Integer = i32;
pub type UInteger = u32;
pub type Long = i64;
pub type ULong = u64;
pub type Byte = i8;
pub type UnsignedByte = u8;
pub type Void = ();
pub type Short = i16;

pub type VecRef<T> = Option<Arc<Vec<T>>>;

pub fn throw<T, E: Error + Send + Sync + 'static>(e: E) -> Result<T> {
    core::result::Result::Err(e)?
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

pub trait Optional {
    fn isNone(&self) -> bool {
        !self.isSome()
    }

    fn isSome(&self) -> bool;
}

impl<T> Optional for Option<T> {
    fn isSome(&self) -> bool {
        self.is_some()
    }
}

pub trait Nullable<T> {
    fn getNull() -> Option<T>;
}

impl<T> Nullable<T> for Option<T> {
    fn getNull() -> Option<T> {
        None
    }
}

pub trait Downgrade<O, R> {
    type WeakAllType: Upgrade<O> + IntoWeak<R>;

    fn downgrade(&self) -> Self::WeakAllType;
}

impl<O, T: ?Sized, R> Downgrade<O, R> for Option<Arc<T>> where Option<Arc<T>>: IntoOriginal<O>,
                                                               Option<Weak<T>>: IntoWeak<R> {
    // 下边的这个原因是 WeakAllType 需要满足 IntoWeak<R>
    type WeakAllType = Option<Weak<T>>;

    fn downgrade(&self) -> Self::WeakAllType {
        if self.is_none() {
            None
        } else {
            Some(Arc::downgrade(self.as_ref().unwrap()))
        }
    }
}

pub trait Upgrade<O> {
    type ArcAllType: IntoOriginal<O>;

    fn upgrade(&self) -> Self::ArcAllType;
}

impl<O, T: ?Sized> Upgrade<O> for Option<Weak<T>> where Option<Arc<T>>: IntoOriginal<O> {
    // 原因 ArcAllType 需要满足 IntoOriginal<O>
    type ArcAllType = Option<Arc<T>>;

    fn upgrade(&self) -> Self::ArcAllType {
        if self.is_none() {
            None
        } else {
            self.as_ref().unwrap().upgrade()
        }
    }
}

pub trait IntoWeak<T>: Sized {
    fn intoWeak(self) -> T;
}

impl<T: ?Sized> IntoWeak<Option<Weak<T>>> for Option<Weak<T>> {
    fn intoWeak(self) -> Option<Weak<T>> {
        self
    }
}

pub trait IntoOriginal<T>: Sized {
    fn intoOriginal(self) -> T;
}

impl<T: ?Sized> IntoOriginal<Option<Arc<T>>> for Option<Arc<T>> {
    fn intoOriginal(self) -> Option<Arc<T>> {
        self
    }
}