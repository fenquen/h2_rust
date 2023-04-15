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
    (pub enum $name:ident {
        $($variant:ident),*,
    }) => {
       pub enum $name {
            $($variant),*
        }

        impl $name {
           pub fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
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

/// user 是 ident
///
/// this.user 是 expr
#[macro_export]
macro_rules! use_ref {

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
macro_rules! use_ref_mut {
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

        use_ref!(user,show);
        use_ref_mut!(user,change,"name0");
        use_ref!(user,show);
    }
}