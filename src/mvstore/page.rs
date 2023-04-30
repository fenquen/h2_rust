use std::cell::RefCell;
use anyhow::Result;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use lazy_static::lazy_static;
use crate::engine::constant;
use crate::{h2_rust_cell_call, h2_rust_cell_mut_call, get_ref, get_ref_mut, suffix_plus_plus, build_option_arc_h2RustCell};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::h2_rust_type::H2RustType::Null;
use crate::h2_rust_common::{Integer, Long};
use crate::mvstore::data_utils;
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

pub type PageTraitSharedPtr = Option<Arc<H2RustCell<dyn PageTrait>>>;

type PageRef = Option<Arc<H2RustCell<Page>>>;

pub trait PageTrait {
    /// 父类实现
    fn init_memory_account(&mut self, memory_count: Integer);

    /// 父类实现
    fn binary_search(&mut self, key: &H2RustType) -> Integer;

    /// 父类实现
    fn get_key_count(&self) -> Integer;

    /// 父类实现 直接在trait实现
    fn isLeaf(&self) -> bool {
        self.getNodeType() == data_utils::PAGE_TYPE_LEAF
    }

    /// abstract
    fn getNodeType(&self) -> Integer;

    /// abstract
    fn getValue(&self, index: Integer) -> H2RustType;

    /// abstract
    fn getChildPage(&self, index: Integer) -> PageTraitSharedPtr;

    /// Get the total number of key-value pairs, including child pages.
    fn getTotalCount(&self) -> Long;

    /// 父类实现
    fn getPosition(&self) -> Long;
}

#[derive(Default)]
pub struct Page {
    pub mv_map: MVMapSharedPtr,

    /// The estimated memory used in persistent case, IN_MEMORY marker value otherwise.
    pub memory: Integer,
    pub keys: Vec<H2RustType>,

    /// The last result of a find operation is cached.
    pub cached_compare: Integer,

    pub position: AtomicI64,
}

impl Page {
    pub fn create_empty_leaf(mv_map_ref: MVMapSharedPtr) -> PageTraitSharedPtr {
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
                       memory: Integer) -> PageTraitSharedPtr {
        assert!(mv_map_ref.is_some());

        let mut leaf = Leaf::new(build_option_arc_h2RustCell!(Page::default()), mv_map_ref, keys, values);
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

pub fn get(mut pageTraitSharedPtr: PageTraitSharedPtr, key: &H2RustType) -> H2RustType {
    loop {
        let pageTraitMutRef = get_ref_mut!(pageTraitSharedPtr);
        let mut index = pageTraitMutRef.binary_search(key);

        if pageTraitMutRef.isLeaf() {
            return if index >= 0 {
                pageTraitMutRef.getValue(index)
            } else {
                Null
            };
        }

        if suffix_plus_plus!(index) < 0 {
            index = -index;
        }

        pageTraitSharedPtr = pageTraitMutRef.getChildPage(index);
    }
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

    fn binary_search(&mut self, key: &H2RustType) -> Integer {
        let mv_map = get_ref!(self.mv_map);
        let ket_type = mv_map.get_key_type();
        let res = ket_type.binary_search(key, &self.keys, self.get_key_count(), self.cached_compare);
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

    fn getNodeType(&self) -> Integer {
        unimplemented!("abstract 需要由子类实现")
    }

    fn getValue(&self, index: Integer) -> H2RustType {
        unimplemented!("abstract 需要由子类实现")
    }

    fn getChildPage(&self, index: Integer) -> PageTraitSharedPtr {
        unimplemented!("abstract 需要由子类实现")
    }

    fn getTotalCount(&self) -> Long {
        unimplemented!("abstract 需要由子类实现")
    }

    fn getPosition(&self) -> Long {
        self.position.load(Ordering::Acquire)
    }
}

pub type LeafRef = Option<Arc<H2RustCell<Leaf>>>;

pub struct Leaf {
    page: PageRef,
    values: Vec<H2RustType>,
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

    fn binary_search(&mut self, key: &H2RustType) -> Integer {
        get_ref_mut!(self.page).binary_search(key)
    }

    fn get_key_count(&self) -> Integer {
        get_ref_mut!(self.page).get_key_count()
    }

    fn getNodeType(&self) -> Integer {
        data_utils::PAGE_TYPE_LEAF
    }

    fn getValue(&self, index: Integer) -> H2RustType {
        match self.values.get(index as usize) {
            Some(h2RustType) => h2RustType.clone(),
            None => Null
        }
    }

    fn getChildPage(&self, index: Integer) -> PageTraitSharedPtr {
        unimplemented!("leaf not support")
    }

    fn getTotalCount(&self) -> Long {
        self.get_key_count() as Long
    }

    fn getPosition(&self) -> Long {
        get_ref!(self.page).getPosition()
    }
}

pub struct NonLeaf {
    page: PageRef,

    /// The child page references.
    children: Vec<PageReferenceSharedPtr>,

    /// The total entry count of this page and all children.
    totalCount: Long,
}

impl NonLeaf {
    fn calculateTotalCount(&self) -> Long {
        let mut totalCount = 0;
        let keyCount = self.get_key_count();
        for a in 0 as usize..keyCount as usize + 1 {
            totalCount = totalCount + get_ref!(self.children.get(a).unwrap()).count;
        }
        totalCount
    }
}

impl PageTrait for NonLeaf {
    fn init_memory_account(&mut self, memory_count: Integer) {
        get_ref_mut!(self.page).init_memory_account(memory_count);
    }

    fn binary_search(&mut self, key: &H2RustType) -> Integer {
        get_ref_mut!(self.page).binary_search(key)
    }

    fn get_key_count(&self) -> Integer {
        get_ref_mut!(self.page).get_key_count()
    }

    fn getNodeType(&self) -> Integer {
        data_utils::PAGE_TYPE_NODE
    }

    fn getValue(&self, index: Integer) -> H2RustType {
        unimplemented!("non leaf not support")
    }

    fn getChildPage(&self, index: Integer) -> PageTraitSharedPtr {
        let pageReferenceRef = get_ref!(self.children.get(index as usize).unwrap());
        let mut pageTraitSharedPtr = pageReferenceRef.page.clone();

        if pageTraitSharedPtr.is_none() {
            let pageRef = get_ref!(self.page);
            let mvMapRef = get_ref!(pageRef.mv_map);

            pageTraitSharedPtr = mvMapRef.read_page(pageRef.mv_map.clone(), pageReferenceRef.position).unwrap();
            assert_eq!(pageReferenceRef.position, get_ref!(pageTraitSharedPtr).getPosition());
            assert_eq!(pageReferenceRef.count, get_ref!(pageTraitSharedPtr).getTotalCount());
        }

        pageTraitSharedPtr
    }

    fn getTotalCount(&self) -> Long {
        assert_eq!(self.totalCount, self.calculateTotalCount());
        self.totalCount
    }

    fn getPosition(&self) -> Long {
        get_ref!(self.page).getPosition()
    }
}

pub type PageReferenceSharedPtr = Option<Arc<H2RustCell<PageReference>>>;

lazy_static! {
    /// Singleton object used when arrays of PageReference have not yet been filled
    static ref EMPTY:PageReferenceSharedPtr = PageReference::new(None, 0, 0);
}
#[derive(Default)]
pub struct PageReference {
    /// The position, if known, or 0.
    pub position: Long,

    /// The page, if in memory, or null.
    pub page: PageTraitSharedPtr,

    /// The descendant count for this child page.
    count: Long,
}

impl PageReference {
    pub fn new(page: PageTraitSharedPtr, position: Long, count: Long) -> PageReferenceSharedPtr {
        build_option_arc_h2RustCell!( PageReference{
            page,
            position,
            count
        })
    }
}