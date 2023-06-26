use crate::api::error_code;

#[macro_export]
// 报错 `impl Trait` in type aliases is unstable
// type Properties = HashMap<String, impl ToString>;
macro_rules! properties_type {
    () => {
        HashMap<String, impl ToString>
    };
}

#[macro_export]
macro_rules! throw {
    ($a:expr) => {
        core::result::Result::Err($a)?
    };
}

#[macro_export]
macro_rules! enum_str {
    (pub enum $type_name:ident {
        $($enum_name:ident),*,
    }) => {
        pub enum $type_name {
            $($enum_name),*
        }

        impl $type_name {
           pub fn name(&self) -> &'static str {
                match self {
                    $($type_name::$enum_name => stringify!($enum_name)),*
                }
            }
        }
    };

     (pub enum $type_name:ident {
        $($enum_name:ident($value:ident)),*,
    }) => {
        pub enum $type_name {
            $($enum_name($value)),*
        }

        impl $type_name {
           pub fn name(&self) -> &'static str {
                match self {
                    $($type_name::$enum_name(_) => stringify!($enum_name)),*
                }
            }
        }
    };

    (pub enum $type_name:ident {
        $($enum_name:ident($value:ty)),*,
    }) => {
        pub enum $type_name {
            $($enum_name($value)),*
        }

        impl $type_name {
           pub fn name(&self) -> &'static str {
                match self {
                    $($type_name::$enum_name(_) => stringify!($enum_name)),*
                }
            }
        }
    };
}

macro_rules! enum_name {
    ($a:expr) => {
        stringify!($a)
    };
}

/// user 是 ident <br>
/// this.user 是 expr
#[macro_export]
macro_rules! atomic_ref_cell {

    ($ref1:ident,$func_name:ident) => {
        {
           let a = $ref1.as_ref().unwrap().borrow();
           let a = a.deref();
           a.$func_name()
        }
    };

    ($ref1:expr,$func_name:ident) => {
        {
           let a = $ref1.as_ref().unwrap().borrow();
           let a = a.deref();
           a.$func_name()
        }
    };

    ($ref1:ident,$func_name:ident,$($variant:expr),*) => {
        {
           let a = $ref1.as_ref().unwrap().borrow();
           let a = a.deref();
           a.$func_name($($variant),*)
        }
    };

    ($ref1:expr,$func_name:ident,$($variant:expr),*) => {
        {
           let a = $ref1.as_ref().unwrap().borrow();
           let a = a.deref();
           a.$func_name($($variant),*)
        }
    };
}

#[macro_export]
macro_rules! atomic_ref_cell_mut {
    ($ref1:ident,$func_name:ident) => {
        {
           let mut a = $ref1.as_ref().unwrap().borrow_mut();
           let a = a.deref_mut();
           a.$func_name()
        }
    };

     ($ref1:expr,$func_name:ident) => {
        {
           let mut a = $ref1.as_ref().unwrap().borrow_mut();
           let a = a.deref_mut();
           a.$func_name()
        }
    };

    ($ref1:ident,$func_name:ident,$($variant:expr),*) => {
        {
           let mut a = $ref1.as_ref().unwrap().borrow_mut();
           let a = a.deref_mut();
           a.$func_name($($variant),*)
        }
    };

    ($ref1:expr,$func_name:ident,$($variant:expr),*) => {
        {
           let mut a = $ref1.as_ref().unwrap().borrow_mut();
           let a = a.deref_mut();
           a.$func_name($($variant),*)
        }
    };
}

#[macro_export]
macro_rules! unsigned_right_shift {
    ($number:expr, $shift:expr, $type_name:ident) => {
        ($number as concat_idents!(U, $type_name) >> $shift) as $type_name
    };
}

#[macro_export]
macro_rules! prefix_plus_plus {
    ($expr:expr) => {
        {
            $expr = $expr + 1;
            $expr
        }
    };
}

#[macro_export]
macro_rules! prefix_minus_minus {
    ($expr:expr) => {
        {
            $expr = $expr - 1;
            $expr
        }
    };
}

#[macro_export]
macro_rules! suffix_plus_plus {
    ($expr:expr) => {
        {
            let old = $expr;
            $expr = $expr + 1;
            old
        }
    };
}

#[macro_export]
macro_rules! suffix_minus_minus {
    ($expr:expr) => {
        {
            let old = $expr;
            $expr = $expr - 1;
            old
        }
    };
}

#[macro_export]
macro_rules! db_error_template {
    ($errorCode:expr, $template:expr, $($variant:expr),*) => {
        DbError::get($errorCode, vec![&format!($template, $($variant),*)])
    };
}

#[macro_export]
macro_rules! load_atomic {
    ($atomic:expr) => {
        $atomic.load(Ordering::Acquired)
    };
}

mod test {
    use std::sync::Arc;
    use atomic_refcell::AtomicRefCell;
    use crate::engine::mode::ModeEnum;

    #[test]
    fn test_throw() {
        // let a =ModeEnum::REGULAR;
        let a = enum_name!(ModeEnum::REGULAR);
        println!("{}", a);
        //crate::throw!(DbError::get(error_code::URL_FORMAT_ERROR_2,vec![engine_constant::URL_FORMAT, &self.url]));
    }

    #[test]
    fn test_expand() {
        use std::ops::{Deref, DerefMut};
        struct User {
            name: String,
        }

        impl User {
            pub fn show(&self) {
                println!("{}", self.name);
            }

            pub fn change(&mut self, name: &str) {
                self.name = name.to_string();
            }
        }

        let user = Some(Arc::new(AtomicRefCell::new(User { name: "name".to_string() })));

        atomic_ref_cell!(user,show);
        atomic_ref_cell_mut!(user,change,"name0");
        atomic_ref_cell!(user,show);
    }
}