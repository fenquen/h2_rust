use std::cell::RefCell;
use std::fmt::Alignment::Left;
use anyhow::Result;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use lazy_static::lazy_static;
use crate::engine::constant;
use crate::{h2_rust_cell_call, h2_rust_cell_mut_call, get_ref, get_ref_mut, suffix_plus_plus, build_option_arc_h2RustCell, throw, db_error_template};
use crate::api::error_code;
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::h2_rust_type::H2RustType::Null;
use crate::h2_rust_common::{Integer, Long, Short};
use crate::h2_rust_common::byte_buffer::ByteBuffer;
use crate::message::db_error::DbError;
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
    fn getKeyCount(&self) -> Integer;

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

    /// 父类实现
    fn setPosition(&self, position: Long);

    /// 父类实现
    fn readFromByteBuffer(&mut self, actual: PageTraitSharedPtr, byteBuffer: &mut ByteBuffer) -> Result<()>;

    /// 父类实现
    fn createKeyStorage(&self, size: Integer) -> Vec<H2RustType>;

    /// 父类实现
    fn createValueStorage(&self, size: Integer) -> Vec<H2RustType>;

    /// abstract
    fn readPayLoad(&mut self, byteBuffer: &mut ByteBuffer);

    /// 父类实现
    fn is_persistent(&self) -> bool;

    /// 父类实现
    fn getMemory(&self) -> Integer;
}

pub type PageSharedPtr = Option<Arc<H2RustCell<Page>>>;

#[derive(Default)]
pub struct Page {
    pub mv_map: MVMapSharedPtr,

    /// The estimated memory used in persistent case, IN_MEMORY marker value otherwise.
    pub memory: Integer,
    pub keys: Vec<H2RustType>,

    /// The last result of a find operation is cached.
    pub cached_compare: Integer,

    pub position: AtomicI64,

    /// 0-based number of the page within containing chunk,默认是-1
    pub pageNo: Integer,

    /// amount of used disk space by this page only in persistent case
    pub diskSpaceUsed: Integer,
}

impl Page {
    pub fn new() -> PageSharedPtr {
        let mut page = Page::default();
        page.pageNo = -1;

        Some(Arc::new(H2RustCell::new(page)))
    }

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

        let mut leaf = Leaf::new(mv_map_ref, keys, values);
        leaf.init_memory_account(memory);
        let page_trait_ref = Arc::new(H2RustCell::new(leaf)) as Arc<H2RustCell<dyn PageTrait>>;
        Some(page_trait_ref)
    }


    fn recalculate_memory(&mut self) {
        assert!(self.is_persistent());
        self.memory = self.calculate_memory();
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
        let keyType = mv_map.getKeyType();
        let res = keyType.binary_search(key, &self.keys, self.getKeyCount(), self.cached_compare);
        self.cached_compare = if res < 0 {
            !res
        } else {
            res + 1
        };
        res
    }

    fn getKeyCount(&self) -> Integer {
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

    fn setPosition(&self, position: Long) {
        self.position.store(position, Ordering::Release);
    }

    fn readFromByteBuffer(&mut self, actual: PageTraitSharedPtr, byteBuffer: &mut ByteBuffer) -> Result<()> {
        let chunkId = data_utils::getPageChunkId(self.position.load(Ordering::Acquire));
        let offset = data_utils::getPageOffset(self.position.load(Ordering::Acquire));

        let start = byteBuffer.getPosition();
        let pageLength = byteBuffer.getI32(); // does not include optional part (pageNo) length是包含自己本身的
        let remaining = byteBuffer.getRemaining() + 4;
        if pageLength as usize > remaining || pageLength < 4 {
            throw!(DbError::get(error_code::FILE_CORRUPTED_1,
                         vec![&format!("file corrupted in chunk {}, expected page length 4..{}, got {}", chunkId, remaining, pageLength)]));
        }

        let check = byteBuffer.getI16();
        let checkTest = data_utils::getCheckValue(chunkId) as Integer
            ^ data_utils::getCheckValue(offset) as Integer
            ^ data_utils::getCheckValue(pageLength) as Integer;
        if check != checkTest as Short {
            throw!(db_error_template!(error_code::FILE_CORRUPTED_1, "file corrupted in chunk {}, expected check value {}, got {}", chunkId, checkTest, check));
        }

        // pageNo
        self.pageNo = data_utils::readVarInt(byteBuffer);
        if self.pageNo < 0 {
            throw!(db_error_template!(error_code::FILE_CORRUPTED_1, "file corrupted in chunk {}, got negative page No {}", chunkId, self.pageNo));
        }

        // mapId
        let mvMapId = data_utils::readVarInt(byteBuffer);
        if mvMapId != get_ref!(self.mv_map).getId() {
            throw!(db_error_template!(error_code::FILE_CORRUPTED_1, "file corrupted in chunk {}, expected map id {}, got {}", chunkId, get_ref!(self.mv_map).getId(), mvMapId));
        }

        // keyCount
        let keyCount = data_utils::readVarInt(byteBuffer);
        self.keys = self.createKeyStorage(keyCount);
        let type7 = byteBuffer.getI8() as Integer;
        if self.isLeaf() != ((type7 & 1) == data_utils::PAGE_TYPE_LEAF) {
            throw!(db_error_template!(error_code::FILE_CORRUPTED_1, "file corrupted in chunk {}, expected node type {}, got {}", chunkId, if self.isLeaf() {"0"} else {"1"}, type7 ));
        }

        byteBuffer.setLimit(start + pageLength as usize);


        if !self.isLeaf() {
            // 需要由下边的实现子类来具体实现 抽象level里又涉及到子类的具体
            // 虚实的结和
            get_ref_mut!(actual).readPayLoad(byteBuffer);
        }

        // todo rust略过压缩

        get_ref!(self.mv_map).getKeyType().read_3(byteBuffer, &mut self.keys, keyCount);

        if self.isLeaf() {
            get_ref_mut!(actual).readPayLoad(byteBuffer);
        }

        self.diskSpaceUsed = pageLength;

        self.recalculate_memory();

        Ok(())
    }

    fn createKeyStorage(&self, size: Integer) -> Vec<H2RustType> {
        let keyType = get_ref!(self.mv_map).getKeyType();
        keyType.create_storage(size)
    }

    fn createValueStorage(&self, size: Integer) -> Vec<H2RustType> {
        let valueType = get_ref!(self.mv_map).getValueType();
        valueType.create_storage(size)
    }

    fn readPayLoad(&mut self, byteBuffer: &mut ByteBuffer) {
        unimplemented!("abstract 需要由子类实现")
    }

    fn is_persistent(&self) -> bool {
        self.memory != IN_MEMORY
    }

    fn getMemory(&self) -> Integer {
        if self.is_persistent() {
            self.memory
        } else {
            0
        }
    }
}

pub type LeafRef = Option<Arc<H2RustCell<Leaf>>>;

#[derive(Default)]
pub struct Leaf {
    page: PageRef,
    values: Vec<H2RustType>,
}

impl Leaf {
    pub fn new(mv_map_ref: MVMapSharedPtr,
               keys: Vec<H2RustType>,
               values: Vec<H2RustType>) -> Leaf {
        let page_ref = Page::new();

        {
            let pageMutRef = get_ref_mut!(page_ref);
            pageMutRef.mv_map = mv_map_ref;
            pageMutRef.keys = keys;
        }

        Leaf {
            page: page_ref,
            values,
        }
    }

    pub fn new1(mvMapSharedPtr: MVMapSharedPtr) -> Leaf {
        let pageRef = build_option_arc_h2RustCell!(Page::default());

        {
            let pageMutRef = pageRef.as_ref().unwrap().get_ref_mut();
            pageMutRef.mv_map = mvMapSharedPtr;
        }

        let mut leaf = Leaf::default();
        leaf.page = pageRef;

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

    fn getKeyCount(&self) -> Integer {
        get_ref_mut!(self.page).getKeyCount()
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
        self.getKeyCount() as Long
    }

    fn getPosition(&self) -> Long {
        get_ref!(self.page).getPosition()
    }

    fn setPosition(&self, position: Long) {
        get_ref!(self.page).setPosition(position);
    }

    fn readFromByteBuffer(&mut self, actual: PageTraitSharedPtr, byteBuffer: &mut ByteBuffer) -> Result<()> {
        get_ref_mut!(self.page).readFromByteBuffer(actual, byteBuffer)
    }

    fn createKeyStorage(&self, size: Integer) -> Vec<H2RustType> {
        get_ref!(self.page).createKeyStorage(size)
    }

    fn createValueStorage(&self, size: Integer) -> Vec<H2RustType> {
        get_ref!(self.page).createValueStorage(size)
    }

    fn readPayLoad(&mut self, byteBuffer: &mut ByteBuffer) {
        let keyCount = self.getKeyCount();
        self.values = self.createValueStorage(keyCount);

        let page = get_ref!(self.page);

        get_ref!(page.mv_map).getValueType().read_3(byteBuffer, &mut self.values, keyCount);
    }

    fn is_persistent(&self) -> bool {
        get_ref!(self.page).is_persistent()
    }

    fn getMemory(&self) -> Integer {
        get_ref!(self.page).getMemory()
    }
}

#[derive(Default)]
pub struct NonLeaf {
    page: PageRef,

    /// The child page references.
    children: Vec<PageReferenceSharedPtr>,

    /// The total entry count of this page and all children.
    totalCount: Long,
}

impl NonLeaf {
    pub fn new1(mvMapSharedPtr: MVMapSharedPtr) -> NonLeaf {
        let pageRef = Page::new();

        {
            let pageMutRef = get_ref_mut!(pageRef);
            pageMutRef.mv_map = mvMapSharedPtr;
        }

        let mut nonLeaf = NonLeaf::default();
        nonLeaf.page = pageRef;

        nonLeaf
    }

    fn calculateTotalCount(&self) -> Long {
        let mut totalCount = 0;
        let keyCount = self.getKeyCount();
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

    fn getKeyCount(&self) -> Integer {
        get_ref_mut!(self.page).getKeyCount()
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


    fn setPosition(&self, position: Long) {
        get_ref!(self.page).setPosition(position);
    }

    fn readFromByteBuffer(&mut self, actual: PageTraitSharedPtr, byteBuffer: &mut ByteBuffer) -> Result<()> {
        get_ref_mut!(self.page).readFromByteBuffer(actual, byteBuffer)
    }

    fn createKeyStorage(&self, size: Integer) -> Vec<H2RustType> {
        get_ref!(self.page).createKeyStorage(size)
    }

    fn createValueStorage(&self, size: Integer) -> Vec<H2RustType> {
        todo!()
    }

    fn readPayLoad(&mut self, byteBuffer: &mut ByteBuffer) {
        let keyCount = get_ref!(self.page).getKeyCount();
        self.children = createRefStorage((keyCount + 1) as usize);

        let mut positions = Vec::with_capacity(keyCount as usize + 1);
        for a in 0..keyCount as usize {
            positions[a] = byteBuffer.getI64();
        }

        let mut total: i64 = 0;

        for a in 0..keyCount as usize {
            let count = data_utils::readVarLong(byteBuffer);

            let position = positions[a];
            if position == 0 {
                assert_eq!(count, 0);
            } else {
                assert!(count >= 0);
            }

            total += count;

            self.children[a] = if position == 0 {
                PageReference::empty()
            } else {
                PageReference::new2(position, count)
            };
        }

        self.totalCount = total;
    }

    fn is_persistent(&self) -> bool {
        get_ref!(self.page).is_persistent()
    }

    fn getMemory(&self) -> Integer {
        get_ref!(self.page).getMemory()
    }
}

pub type PageReferenceSharedPtr = Option<Arc<H2RustCell<PageReference>>>;

lazy_static! {
    /// Singleton object used when arrays of PageReference have not yet been filled
    static ref EMPTY:PageReferenceSharedPtr = PageReference::new3(None, 0, 0);
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
    pub fn new3(page: PageTraitSharedPtr, position: Long, count: Long) -> PageReferenceSharedPtr {
        build_option_arc_h2RustCell!(PageReference{
            page,
            position,
            count
        })
    }

    pub fn new2(position: Long, count: Long) -> PageReferenceSharedPtr {
        build_option_arc_h2RustCell!(PageReference{
            page:None,
            position,
            count
        })
    }
    pub fn empty() -> PageReferenceSharedPtr {
        EMPTY.clone()
    }
}

pub fn readFromByteBuffer(byteBuffer: &mut ByteBuffer, position: Long, mvMap: MVMapSharedPtr) -> Result<PageTraitSharedPtr> {
    let isLeaf = (data_utils::getPageType(position) & 1) == data_utils::PAGE_TYPE_LEAF;

    let pageTrait = if isLeaf {
        let leaf = Leaf::new1(mvMap);
        Some(Arc::new(H2RustCell::new(leaf)) as Arc<H2RustCell<dyn PageTrait>>)
    } else {
        let nonLeaf = NonLeaf::new1(mvMap);
        Some(Arc::new(H2RustCell::new(nonLeaf)) as Arc<H2RustCell<dyn PageTrait>>)
    };

    get_ref!(pageTrait).setPosition(position);

    get_ref_mut!(pageTrait).readFromByteBuffer(pageTrait.clone(), byteBuffer)?;

    Ok(pageTrait)
}

pub fn createRefStorage(size: usize) -> Vec<PageReferenceSharedPtr> {
    Vec::with_capacity(size)
}
