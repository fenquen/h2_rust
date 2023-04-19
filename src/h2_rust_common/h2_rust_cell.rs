use std::cell::UnsafeCell;

pub struct H2RustCell<T: ?Sized> {
    data: UnsafeCell<T>,
}

impl<T> H2RustCell<T> {
    pub fn new(t: T) -> H2RustCell<T> {
        H2RustCell {
            data: UnsafeCell::new(t)
        }
    }
}

impl<T: ?Sized> H2RustCell<T> {
    pub fn get_ref(&self) -> &T {
        unsafe { &*self.data.get() }
    }

    pub fn get_ref_mut(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

unsafe impl<T: ?Sized> Send for H2RustCell<T> {}

unsafe impl<T: ?Sized> Sync for H2RustCell<T> {}
/*impl<T> Drop for Wrapper<T> {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        unsafe {
            ptr::drop_in_place(self.0);
            alloc::dealloc(self.0 as *mut u8, Layout::new::<T>());
        }
    }
}*/

#[macro_export]
macro_rules! h2_rust_cell_call {
    ($h2_rust_cell_option:ident, $func_name:ident) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref().$func_name()
        }
    };

    ($h2_rust_cell_option:expr, $func_name:ident) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref().$func_name()
        }
    };

    ($h2_rust_cell_option:ident, $func_name:ident, $($variant:expr),*) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref().$func_name($($variant),*)
        }
    };

    ($h2_rust_cell_option:expr, $func_name:ident, $($variant:expr),*) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref().$func_name($($variant),*)
        }
    }
}

#[macro_export]
macro_rules! h2_rust_cell_mut_call {
    ($h2_rust_cell_option:ident, $func_name:ident) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref_mut().$func_name()
        }
    };

    ($h2_rust_cell_option:expr, $func_name:ident) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref_mut().$func_name()
        }
    };

    ($h2_rust_cell_option:ident, $func_name:ident, $($variant:expr),*) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref_mut().$func_name($($variant),*)
        }
    };

    ($h2_rust_cell_option:expr, $func_name:ident, $($variant:expr),*) => {
        {
            $h2_rust_cell_option.as_ref().unwrap().get_ref_mut().$func_name($($variant),*)
        }
    };
}

#[macro_export]
macro_rules! h2_rust_cell_ref {
    ($h2_rust_cell_option:ident) => {
        $h2_rust_cell_option.as_ref().unwrap().get_ref()
    };

    ($h2_rust_cell_option:expr) => {
        $h2_rust_cell_option.as_ref().unwrap().get_ref()
    };
}

#[macro_export]
macro_rules! h2_rust_cell_ref_mutable {
    ($h2_rust_cell_option:ident) => {
        $h2_rust_cell_option.as_ref().unwrap().get_ref_mut()
    };

    ($h2_rust_cell_option:expr) => {
        $h2_rust_cell_option.as_ref().unwrap().get_ref_mut()
    };
}

#[macro_export]
macro_rules! build_h2_rust_cell {
    ($ident:ident) => {
        Some(Arc::new(H2RustCell::new($ident)))
    };

    ($expr:expr) => {
        Some(Arc::new(H2RustCell::new($expr)))
    }
}