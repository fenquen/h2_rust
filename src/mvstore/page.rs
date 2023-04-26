use std::cell::RefCell;
use anyhow::Result;
use std::ops::Deref;
use std::sync::Arc;
use crate::engine::constant;
use crate::{h2_rust_cell_call, h2_rust_cell_mut_call, get_ref, get_ref_mut};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::Integer;
use crate::mvstore::mv_map::MVMapSharedPtr;

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

pub type PageTraitRef = Option<Arc<H2RustCell<dyn PageTrait>>>;

type PageRef = Option<Arc<H2RustCell<Page>>>;

pub trait PageTrait {
    fn init_memory_account(&mut self, memory_count: Integer);

    fn binary_search(&mut self, key: H2RustType) -> Integer;

    fn get_key_count(&self) -> Integer;
}

#[derive(Default)]
pub struct Page {
    pub mv_map: MVMapSharedPtr,

    /// The estimated memory used in persistent case, IN_MEMORY marker value otherwise.
    memory: Integer,
    keys: Vec<H2RustType>,

    /// The last result of a find operation is cached.
    cached_compare: Integer,
}

impl Page {
    pub fn create_empty_leaf(mv_map_ref: MVMapSharedPtr) -> PageTraitRef {
        let mv_map = get_ref!(mv_map_ref);

        let keys = mv_map.key_type.as_ref().unwrap().create_storage(0);
        let values = mv_map.value_type.as_ref().unwrap().create_storage(0);

        //drop(mv_map_atomic_ref);

        Self::create_leaf(mv_map_ref,
                          keys,
                          values,
                          PAGE_LEAF_MEMORY)
    }

    pub fn create_leaf(mv_map_ref: MVMapSharedPtr,
                       keys: Vec<H2RustType>,
                       values: Vec<H2RustType>,
                       memory: Integer) -> PageTraitRef {
        assert!(mv_map_ref.is_some());
        let page_ref = Some(Arc::new(H2RustCell::new(Page::default())));
        let mut leaf = Leaf::new(page_ref, mv_map_ref, keys, values);
        leaf.init_memory_account(memory);
        let page_trait_ref = Arc::new(H2RustCell::new(leaf)) as Arc<H2RustCell<dyn PageTrait>>;
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

pub fn get(page_trait_ref: PageTraitRef, key: H2RustType) -> H2RustType {
    let index = get_ref_mut!(page_trait_ref).binary_search(key);
    todo!()
}

impl PageTrait for Page {
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

    fn binary_search(&mut self, key: H2RustType) -> Integer {
        let mv_map = get_ref!(self.mv_map);
        let ket_type = mv_map.get_key_type();
        let res = ket_type.binary_search(&key, &self.keys, self.get_key_count(), self.cached_compare);
        self.cached_compare = if res < 0 {
            !res
        } else {
            res + 1
        };
        res
    }

    fn get_key_count(&self) -> Integer {
        self.keys.len() as Integer
    }
}

pub type LeafRef = Option<Arc<H2RustCell<Leaf>>>;

pub struct Leaf {
    pub page: PageRef,
    pub values: Vec<H2RustType>,
}

impl Leaf {
    pub fn new(page_ref: PageRef,
               mv_map_ref: MVMapSharedPtr,
               keys: Vec<H2RustType>,
               values: Vec<H2RustType>) -> Leaf {
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

impl PageTrait for Leaf {
    fn init_memory_account(&mut self, memory_count: Integer) {
        get_ref_mut!(self.page).init_memory_account(memory_count);
    }

    fn binary_search(&mut self, key: H2RustType) -> Integer {
        get_ref_mut!(self.page).binary_search(key)
    }

    fn get_key_count(&self) -> Integer {
        todo!()
    }
}