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

mod test{
    use crate::engine::mode::ModeEnum;

    #[test]
    fn test_throw(){
       // let a =ModeEnum::REGULAR;
        let a = enum_name!(ModeEnum::REGULAR);
        println!("{}",a);
        //crate::throw!(DbError::get(error_code::URL_FORMAT_ERROR_2,vec![engine_constant::URL_FORMAT, &self.url]));
    }
}