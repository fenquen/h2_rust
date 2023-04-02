use std::collections::HashMap;

#[derive(Default)]
pub struct MVStore {}

impl MVStore {}

#[derive(Default)]
pub struct MVStoreBuilder {
    pub config: HashMap<String, String>,
}

impl MVStoreBuilder {
    pub fn new() -> Self {
        MVStoreBuilder::default()
    }

}
