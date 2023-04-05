use std::cmp;
use std::env::var;
use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::data_utils;
use crate::mvstore::page::Page;

#[derive(Default)]
pub struct CacheLongKeyLIRS<V> {
    /// the maximum memory this cache should use.
    max_memory: Long,
    segment_arr: Nullable<Vec<Segment<V>>>,
    segment_count: Integer,
    segment_shift: Integer,
    segment_mask: Integer,
    stack_move_distance: Integer,
    non_resident_queue_size: Integer,
    non_resident_queue_size_high: Integer,
}

impl<V: Default + Clone> CacheLongKeyLIRS<V> {
    pub fn new(config: &CacheLongKeyLIRSConfig) -> CacheLongKeyLIRS<V> {
        let mut cache_long_key_lirs = CacheLongKeyLIRS::default();
        cache_long_key_lirs.init(config);
        cache_long_key_lirs
    }

    pub fn init(&mut self, config: &CacheLongKeyLIRSConfig) {
        data_utils::check_argument(Integer::count_ones(config.segment_count) == 1, "segment count must be a power of 2");
        self.segment_count = config.segment_count;

        self.set_max_memory(config.max_memory);
        self.non_resident_queue_size = config.non_resident_queue_size;
        self.non_resident_queue_size_high = config.non_resident_queue_size_high;

        self.segment_mask = self.segment_count - 1;
        self.stack_move_distance = config.stack_move_distance;
        self.segment_arr = NotNull(Vec::<Segment<V>>::with_capacity(self.segment_count as usize));

        self.clear();

        // use the high bits for the segment
        self.segment_shift = (32 - Integer::count_ones(self.segment_mask)) as Integer;
    }

    pub fn set_max_memory(&mut self, max_memory: Long) {
        data_utils::check_argument(max_memory > 0, "Max memory must be larger than 0");
        self.max_memory = max_memory;

        if !self.segment_arr.is_null() {
            let segment_arr = self.segment_arr.unwrap_mut();
            let max = 1 + max_memory / segment_arr.len() as Long;
            for segment in segment_arr {
                segment.max_memory = max;
            }
        }
    }

    pub fn clear(&mut self) {
        let max = self.get_max_item_size();
        let segment_arr = self.segment_arr.unwrap_mut();
        for _ in 0..self.segment_count {
            segment_arr.push(Segment::<V>::new(
                max,
                self.stack_move_distance,
                8, self.non_resident_queue_size,
                self.non_resident_queue_size_high));
        }
    }

    pub fn get_max_item_size(&self) -> Long {
        cmp::max(1, self.max_memory / self.segment_count as Long)
    }

    pub fn put(&mut self, key: Long, value: V, memory: Integer) {}
}

#[derive(Default)]
pub struct Segment<V> {
    /// The number of (hot, cold, and non-resident) entries in the map.
    map_size: Integer,

    /// The size of the LIRS queue for resident cold entries.
    queue_size: Integer,

    /// The size of the LIRS queue for non-resident cold entries.
    queue2size: Integer,

    /// The number of cache hits.
    hits: Long,

    /// The number of cache misses.
    misses: Long,

    /// The map array. The size is always a power of 2.
    entries: Vec<EntryRef<V>>,

    /// The currently used memory.
    used_memory: Long,

    /// How many other item are to be moved to the top of the stack before the current item is moved.
    stack_move_distance: Integer,

    /// Set the maximum memory this cache should use. This will not
    /// immediately cause entries to get removed however; it will only change
    /// the limit. To resize the internal array, call the clear method.
    /// the maximum size (1 or larger) this cache should use in bytes
    max_memory: Long,

    /// The bit mask that is applied to the key hash code to get the index in
    /// the map array. The mask is the length of the array minus one.
    mask: Integer,

    /// Low watermark for the number of entries in the non-resident queue,
    /// as a factor of the number of entries in the map.
    non_resident_queue_size: Integer,

    /// High watermark for the number of entries in the non-resident queue, as a factor of the number of entries in the map.
    non_resident_queue_size_high: Integer,

    /// The stack of recently referenced elements. This includes all hot
    /// entries, and the recently referenced cold entries. Resident cold
    /// entries that were not recently referenced, as well as non-resident cold entries, are not in the stack.
    ///
    /// There is always at least one entry: the head entry.
    stack: EntryRef<V>,

    /// The number of entries in the stack.
    stack_size: Integer,

    /// The queue of resident cold entries.
    ///
    /// There is always at least one entry: the head entry.
    queue: EntryRef<V>,

    /// The queue of non-resident cold entries.
    ///
    /// There is always at least one entry: the head entry.
    queue2: EntryRef<V>,

    /// The number of times any item was moved to the top of the stack.
    stack_move_counter: Integer,
}

pub type SegmentRef<V> = Nullable<Arc<AtomicRefCell<Segment<V>>>>;

impl<V: Default + Clone> Segment<V> {
    pub fn new(max_memory: Long,
               stack_move_distance: Integer,
               len: Integer,
               non_resident_queue_size: Integer,
               non_resident_queue_size_high: Integer) -> Segment<V> {
        Segment::<V>::default()
    }

    pub fn init(&mut self,
                max_memory: Long,
                stack_move_distance: Integer,
                len: Integer,
                non_resident_queue_size: Integer,
                non_resident_queue_size_high: Integer) {
        self.max_memory = max_memory;
        self.stack_move_distance = stack_move_distance;
        self.non_resident_queue_size = non_resident_queue_size;
        self.non_resident_queue_size_high = non_resident_queue_size_high;

        // the bit mask has all bits set
        self.mask = len - 1;

        // initialize the stack and queue heads
        self.stack = Entry::new_0();
        let mut atomic_ref_mut = self.stack.unwrap().borrow_mut();
        atomic_ref_mut.stack_prev = self.stack.clone();
        atomic_ref_mut.stack_next = self.stack.clone();

        self.queue = Entry::new_0();
        let mut atomic_ref_mut = self.queue.unwrap().borrow_mut();
        atomic_ref_mut.queue_prev = self.queue.clone();
        atomic_ref_mut.queue_next = self.queue.clone();

        self.queue2 = Entry::new_0();
        let mut atomic_ref_mut = self.queue2.unwrap().borrow_mut();
        atomic_ref_mut.queue_prev = self.queue2.clone();
        atomic_ref_mut.queue_next = self.queue2.clone();

        self.entries = Vec::with_capacity(len as usize);
    }
}

pub type EntryRef<V> = Nullable<Arc<AtomicRefCell<Entry<V>>>>;

#[derive(Default)]
pub struct Entry<V> {
    /// The key
    key: Long,

    /// The value. Set to null for non-resident-cold entries.
    value: V,

    /// Weak reference to the value. Set to null for resident entries.
    // WeakReference<V> reference;

    /// The estimated memory used.
    memory: Integer,

    /// When the item was last moved to the top of the stack.
    top_move: Integer,

    /// The next entry in the stack.
    stack_next: EntryRef<V>,

    /// The previous entry in the stack.
    stack_prev: EntryRef<V>,

    /// The next entry in the queue (either the resident queue or the non-resident queue).
    queue_next: EntryRef<V>,

    /// The previous entry in the queue.
    queue_prev: EntryRef<V>,

    /// The next entry in the map (the chained entry).
    map_next: EntryRef<V>,
}

impl<V: Clone + Default> Entry<V> {
    pub fn new_0() -> EntryRef<V> {
        NotNull(Arc::new(AtomicRefCell::new(Entry::default())))
    }

    pub fn new_3(key: Long, value: V, memory: Integer) -> EntryRef<V> {
        let mut entry = Entry::default();
        entry.key = key;
        entry.value = value;
        entry.memory = memory;
        NotNull(Arc::new(AtomicRefCell::new(entry)))
    }

    pub fn new_1(old: &EntryRef<V>) -> EntryRef<V> {
        let mut entry = Entry::default();

        let atomic_ref = old.unwrap().borrow();
        let old = atomic_ref;
        entry.key = old.key;
        entry.value = (&*old).value.clone();
        entry.memory = old.memory;
        NotNull(Arc::new(AtomicRefCell::new(entry)))
    }

    /// whether this entry is hot. Cold entries are in one of the two queues.
    pub fn is_hot(&self) -> bool {
        self.queue_next.is_null()
    }

    pub fn get_value(&self) -> V {
        self.value.clone()
    }

    pub fn get_memory(&self) -> Integer {
        self.memory
    }
}

pub struct CacheLongKeyLIRSConfig {
    /// The maximum memory to use (1 or larger).
    pub max_memory: Long,

    /// The number of cache segments (must be a power of 2).
    pub segment_count: Integer,

    /// How many other item are to be moved to the top of the stack before the current item is moved.
    pub stack_move_distance: Integer,

    /// Low water mark for the number of entries in the non-resident queue,
    /// as a factor of the number of all other entries in the map.
    pub non_resident_queue_size: Integer,

    /// High watermark for the number of entries in the non-resident queue,
    /// as a factor of the number of all other entries in the map
    pub non_resident_queue_size_high: Integer,
}

impl CacheLongKeyLIRSConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for CacheLongKeyLIRSConfig {
    fn default() -> Self {
        Self {
            max_memory: 1,
            segment_count: 16,
            stack_move_distance: 32,
            non_resident_queue_size: 3,
            non_resident_queue_size_high: 12,
        }
    }
}