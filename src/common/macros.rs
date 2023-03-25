#[macro_export]
// 报错 `impl Trait` in type aliases is unstable
// type Properties = HashMap<String, impl ToString>;
macro_rules! properties_type {
    () => {
        HashMap<String, impl ToString>
    };
}