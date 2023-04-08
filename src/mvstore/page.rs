use std::ops::Deref;
use crate::engine::constant;
use crate::h2_rust_common::Integer;
use crate::mvstore::mv_map::MVMapRef;

/// The estimated number of bytes used per child entry.
const PAGE_MEMORY_CHILD: Integer = constant::MEMORY_POINTER + 16; //  16 = two longs

/// The estimated number of bytes used per base page.
const PAGE_MEMORY: Integer = constant::MEMORY_OBJECT +           // this
    2 * constant::MEMORY_POINTER +      // map, keys
    constant::MEMORY_ARRAY +            // Object[] keys
    17;                       // pos, cachedCompare, memory, removedInMemory

/// The estimated number of bytes used per empty internal page object.
const PAGE_NODE_MEMORY: Integer = PAGE_MEMORY +             // super
    constant::MEMORY_POINTER +          // children
    constant::MEMORY_ARRAY +            // Object[] children
    8;                        // totalCount

/// The estimated number of bytes used per empty leaf page.
const PAGE_LEAF_MEMORY: Integer = PAGE_MEMORY +  // super
    constant::MEMORY_POINTER +          // values
    constant::MEMORY_ARRAY;             // Object[] values

#[derive(Default, Clone)]
pub struct Page<K, V> {
    pub mv_map: MVMapRef<K, V>,
}

impl<K, V> Page<K, V> {
    pub fn create_empty_leaf<K1, V1>(mv_map_ref: MVMapRef<K1, V1>) -> Page<K1, V1> {
        let mv_map_atomic_ref = mv_map_ref.as_ref().unwrap().borrow();
        let mv_map = mv_map_atomic_ref.deref();

        let keys=mv_map.key_type.as_ref().unwrap().create_storage(0);
        let values = mv_map.value_type.as_ref().unwrap().create_storage(0);

        drop(mv_map_atomic_ref);

        Self::create_leaf(mv_map_ref,
                          keys,
                          values,
                          PAGE_LEAF_MEMORY)
    }

    pub fn create_leaf<K1, V1>(mv_map_ref: MVMapRef<K1, V1>,
                             keys: Vec<K1>,
                             values: Vec<V1>,
                             memory: Integer) -> Page<K1, V1> {
       todo!()
    }
}