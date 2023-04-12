use std::cell::RefCell;
use anyhow::Result;
use std::ops::Deref;
use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
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
    constant::MEMORY_ARRAY; // Object[] values

const IN_MEMORY: Integer = Integer::MIN;

pub type PageTraitRef<K, V> = Option<Arc<AtomicRefCell<dyn PageTrait<K, V>>>>;

pub type PageRef<K, V> = Option<Arc<AtomicRefCell<Page<K, V>>>>;

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
    pub fn create_empty_leaf<K1: Default + 'static, V1: Default + 'static>(mv_map_ref: MVMapRef<K1, V1>) -> PageTraitRef<K1, V1> {
        let mv_map_atomic_ref = mv_map_ref.as_ref().unwrap().borrow();
        let mv_map = mv_map_atomic_ref.deref();

        let keys = mv_map.key_type.as_ref().unwrap().create_storage(0);
        let values = mv_map.value_type.as_ref().unwrap().create_storage(0);

        drop(mv_map_atomic_ref);

        Self::create_leaf(mv_map_ref,
                          keys,
                          values,
                          PAGE_LEAF_MEMORY)
    }

    pub fn create_leaf<K1: Default + 'static, V1: Default + 'static>(mv_map_ref: MVMapRef<K1, V1>,
                                                                     keys: Vec<K1>,
                                                                     values: Vec<V1>,
                                                                     memory: Integer) -> PageTraitRef<K1, V1> {
        assert!(mv_map_ref.is_some());
        let page_ref = Some(Arc::new(AtomicRefCell::new(Page::<K1, V1>::default())));
        let mut page = Leaf::new(page_ref, mv_map_ref, keys, values);
        page.init_memory_account(memory);
        let page_trait_ref = Arc::new(AtomicRefCell::new(page)) as Arc<AtomicRefCell<dyn PageTrait<K1, V1>>>;
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

impl<K: Default+ 'static, V: Default+ 'static> PageTrait<K, V> for Page<K, V> {
    /// 要在trait上
    fn init_memory_account(&mut self, memory_count: Integer) {
        if !self.mv_map.as_ref().unwrap().borrow().is_persistent() {
            self.memory = IN_MEMORY;
        } else if memory_count == 0 {
            self.recalculate_memory();
        } else {
            self.add_memory(memory_count);
            assert_eq!(memory_count, self.get_memory());
        }
    }
}

pub type LeafRef<K, V> = Option<Arc<AtomicRefCell<Leaf<K, V>>>>;

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
            let mut atomic_ref_mut = page_ref.as_ref().unwrap().borrow_mut();
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

impl<K: Default+ 'static, V: Default+ 'static> PageTrait<K, V> for Leaf<K, V> {
    fn init_memory_account(&mut self, memory_count: Integer) {
        self.page.as_ref().unwrap().borrow_mut().init_memory_account(memory_count);
    }
}