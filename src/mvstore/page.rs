use crate::mvstore::mv_map::MVMapRef;

#[derive(Default)]
pub struct Page<K, V> {
    pub mv_map: MVMapRef<K, V>,
}