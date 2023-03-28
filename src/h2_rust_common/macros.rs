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

mod test{
    #[test]
    fn test_throw(){
        //crate::throw!(DbError::get(error_code::URL_FORMAT_ERROR_2,vec![engine_constant::URL_FORMAT, &self.url]));
    }
}