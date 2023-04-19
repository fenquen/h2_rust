use std::cmp;
use std::env::var;
use std::sync::Arc;
use crate::{build_h2_rust_cell, h2_rust_cell_ref, h2_rust_cell_ref_mutable};
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::data_utils;
use crate::mvstore::page::Page;

#[derive(Default)]
pub struct CacheLongKeyLIRS<V> {
    /// the maximum memory this cache should use.
    max_memory: Long,
    segment_arr: Option<Vec<Segment<V>>>,
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
        self.segment_arr = Some(Vec::<Segment<V>>::with_capacity(self.segment_count as usize));

        self.clear();

        // use the high bits for the segment
        self.segment_shift = (32 - Integer::count_ones(self.segment_mask)) as Integer;
    }

    pub fn set_max_memory(&mut self, max_memory: Long) {
        data_utils::check_argument(max_memory > 0, "Max memory must be larger than 0");
        self.max_memory = max_memory;

        if self.segment_arr.is_some() {
            let segment_arr = self.segment_arr.as_mut().unwrap();
            let max = 1 + max_memory / segment_arr.len() as Long;
            for segment in segment_arr {
                segment.max_memory = max;
            }
        }
    }

    pub fn clear(&mut self) {
        let max = self.get_max_item_size();
        let segment_arr = self.segment_arr.as_mut().unwrap();
        for _ in 0..self.segment_count {
            segment_arr.push(Segment::<V>::new(
                max,
                self.stack_move_distance,
                8, self.non_resident_queue_size,
                self.non_resident_queue_size_high));
        }
    }

    /// determines max size of the data item size to fit into cache
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

pub type SegmentRef<V> = Option<Arc<H2RustCell<Segment<V>>>>;

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
        let ref_mut = h2_rust_cell_ref_mutable!(self.stack);
        ref_mut.stack_prev = self.stack.clone();
        ref_mut.stack_next = self.stack.clone();

        self.queue = Entry::new_0();
        let mut ref_mut = h2_rust_cell_ref_mutable!(self.queue);
        ref_mut.queue_prev = self.queue.clone();
        ref_mut.queue_next = self.queue.clone();

        self.queue2 = Entry::new_0();
        let mut ref_mut = h2_rust_cell_ref_mutable!(self.queue2);
        ref_mut.queue_prev = self.queue2.clone();
        ref_mut.queue_next = self.queue2.clone();

        self.entries = Vec::with_capacity(len as usize);
    }
}

pub type EntryRef<V> = Option<Arc<H2RustCell<Entry<V>>>>;

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

impl<V: Default + Clone> Entry<V> {
    pub fn new_0() -> EntryRef<V> {
        build_h2_rust_cell!(Entry::default())
    }

    pub fn new_3(key: Long, value: V, memory: Integer) -> EntryRef<V> {
        let mut entry = Entry::default();
        entry.key = key;
        entry.value = value;
        entry.memory = memory;
        build_h2_rust_cell!(entry)
    }

    pub fn new_1(old: &EntryRef<V>) -> EntryRef<V> {
        let mut entry = Entry::default();

        let old = h2_rust_cell_ref!(old);
        entry.key = old.key;
        entry.value = old.value.clone();
        entry.memory = old.memory;
        build_h2_rust_cell!(entry)
    }

    /// whether this entry is hot. Cold entries are in one of the two queues.
    pub fn is_hot(&self) -> bool {
        self.queue_next.is_none()
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