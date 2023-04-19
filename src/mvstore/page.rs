use std::cell::RefCell;
use anyhow::Result;
use std::ops::Deref;
use std::sync::Arc;
use crate::engine::constant;
use crate::{h2_rust_cell_call, h2_rust_cell_mut_call, h2_rust_cell_ref};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
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
    constant::MEMORY_ARRAY; // Object[] values

const IN_MEMORY: Integer = Integer::MIN;

pub type PageTraitRef<K, V> = Option<Arc<H2RustCell<dyn PageTrait<K, V>>>>;

type PageRef<K, V> = Option<Arc<H2RustCell<Page<K, V>>>>;

pub trait PageTrait<K, V> {
    fn init_memory_account(&mut self, memory_count: Integer);
}

#[derive(Default)]
pub struct Page<K, V> {
    pub mv_map: MVMapRef<K, V>,
    /// The estimated memory used in persistent case, IN_MEMORY marker value otherwise.
    memory: Integer,
    keys: Vec<K>,
}

impl<K: Default, V: Default> Page<K, V> {
    pub fn create_empty_leaf<K1, V1>(mv_map_ref: MVMapRef<K1, V1>) -> PageTraitRef<K1, V1> where K1: Default + 'static,
                                                                                                 V1: Default + 'static {
        let mv_map = h2_rust_cell_ref!(mv_map_ref);

        let keys = mv_map.key_type.as_ref().unwrap().create_storage(0);
        let values = mv_map.value_type.as_ref().unwrap().create_storage(0);

        //drop(mv_map_atomic_ref);

        Self::create_leaf(mv_map_ref,
                          keys,
                          values,
                          PAGE_LEAF_MEMORY)
    }

    pub fn create_leaf<K1, V1>(mv_map_ref: MVMapRef<K1, V1>,
                               keys: Vec<K1>,
                               values: Vec<V1>,
                               memory: Integer) -> PageTraitRef<K1, V1> where K1: Default + 'static,
                                                                              V1: Default + 'static {
        assert!(mv_map_ref.is_some());
        let page_ref = Some(Arc::new(H2RustCell::new(Page::<K1, V1>::default())));
        let mut page = Leaf::new(page_ref, mv_map_ref, keys, values);
        page.init_memory_account(memory);
        let page_trait_ref = Arc::new(H2RustCell::new(page)) as Arc<H2RustCell<dyn PageTrait<K1, V1>>>;
        Some(page_trait_ref)
    }


    fn recalculate_memory(&mut self) {
        assert!(self.is_persistent());
        self.memory = self.calculate_memory();
    }

    fn is_persistent(&self) -> bool {
        self.memory != IN_MEMORY
    }

    fn calculate_memory(&self) -> Integer {
        // todo mvMap.evaluateMemoryForKeys(keys, getKeyCount());
        1
    }

    fn add_memory(&mut self, mem: Integer) {
        self.memory = self.memory + mem;
        assert!(0 <= self.memory);
    }

    /// 要在trait上
    pub fn get_memory(&self) -> Integer {
        if self.is_persistent() {
            return self.memory;
        }
        0
    }
}

impl<K, V> PageTrait<K, V> for Page<K, V> where K: Default + 'static,
                                                V: Default + 'static {
    /// 要在trait上
    fn init_memory_account(&mut self, memory_count: Integer) {
        if !h2_rust_cell_call!(self.mv_map, is_persistent) {
            self.memory = IN_MEMORY;
        } else if memory_count == 0 {
            self.recalculate_memory();
        } else {
            self.add_memory(memory_count);
            assert_eq!(memory_count, self.get_memory());
        }
    }
}

pub type LeafRef<K, V> = Option<Arc<H2RustCell<Leaf<K, V>>>>;

pub struct Leaf<K, V> {
    pub page: PageRef<K, V>,
    pub values: Vec<V>,
}

impl<K, V> Leaf<K, V> {
    pub fn new(page_ref: PageRef<K, V>,
               mv_map_ref: MVMapRef<K, V>,
               keys: Vec<K>,
               values: Vec<V>) -> Leaf<K, V> {
        {
            let mut atomic_ref_mut = page_ref.as_ref().unwrap().get_ref_mut();
            atomic_ref_mut.mv_map = mv_map_ref;
            atomic_ref_mut.keys = keys;
        }

        let leaf = Leaf {
            page: page_ref,
            values,
        };

        leaf
    }
}

impl<K, V> PageTrait<K, V> for Leaf<K, V> where K: Default + 'static,
                                                V: Default + 'static {
    fn init_memory_account(&mut self, memory_count: Integer) {
        h2_rust_cell_mut_call!(self.page, init_memory_account, memory_count);
    }
}