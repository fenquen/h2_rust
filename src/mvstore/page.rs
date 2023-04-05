use crate::mvstore::mv_map::MVMapRef;

#[derive(Default,Clone)]
pub struct Page<K, V> {
    pub mv_map: MVMapRef<K, V>,
}