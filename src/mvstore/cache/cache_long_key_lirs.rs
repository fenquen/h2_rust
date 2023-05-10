use std::cmp;
use std::env::var;
use std::ops::Deref;
use std::sync::Arc;
use usync::lock_api::RawMutex;
use usync::ReentrantMutex;
use crate::{build_option_arc_h2RustCell, get_ref, get_ref_mut, h2_rust_cell_equals, suffix_plus_plus, unsigned_right_shift};
use crate::h2_rust_common::{Integer, Long, MyMutex, Nullable, Optional, ULong};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::data_utils;
use crate::mvstore::page::Page;
use crate::h2_rust_common::UInteger;

#[derive(Default)]
pub struct CacheLongKeyLIRS<V> {
    /// the maximum memory this cache should use.
    max_memory: Long,
    segment_arr: Option<Vec<SegmentRef<V>>>,
    segment_count: Integer,
    segment_shift: Integer,
    segment_mask: Integer,
    stack_move_distance: Integer,
    non_resident_queue_size: Integer,
    non_resident_queue_size_high: Integer,
}

impl<V: Default + Clone + Optional> CacheLongKeyLIRS<V> {
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
        self.segment_arr = Some(Vec::<SegmentRef<V>>::with_capacity(self.segment_count as usize));

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
            for segment_ref in segment_arr {
                get_ref_mut!(segment_ref).max_memory = max;
            }
        }
    }

    pub fn clear(&mut self) {
        let max = self.get_max_item_size();
        let segment_arr = self.segment_arr.as_mut().unwrap();
        for _ in 0..self.segment_count {
            segment_arr.push(build_option_arc_h2RustCell!(Segment::<V>::new(
                max,
                self.stack_move_distance,
                8,
                self.non_resident_queue_size,
                self.non_resident_queue_size_high)));
        }
    }

    /// determines max size of the data item size to fit into cache
    pub fn get_max_item_size(&self) -> Long {
        cmp::max(1, self.max_memory / self.segment_count as Long)
    }


    pub fn put(&mut self, key: Long, value: V, memory: Integer) {

    }

    pub fn get(&mut self, key: Long) -> V {
        let hash = Self::get_hash(key);
        let segment_ref = self.get_segment(hash);
        let entry_ref = get_ref!(segment_ref).find(key, hash);
        get_ref_mut!(segment_ref).get(entry_ref) // 因为该函数内部需要V上有Optional相应函数使得CacheLongKeyLIRS的V也要实现Optional,部下污染了上头
    }

    fn get_hash(key: Long) -> Integer {
        let mut hash = ((key as ULong >> 32) as Long ^ key) as Integer;
        // a supplemental secondary hash function to protect against hash codes that don't differ much
        hash = unsigned_right_shift!(hash, 16, Integer);
        hash = (unsigned_right_shift!(hash, 16, Integer) ^ hash) * 0x45d9f3b;
        hash = (unsigned_right_shift!(hash, 16, Integer) ^ hash) * 0x45d9f3b;
        hash = unsigned_right_shift!(hash, 16, Integer) ^ hash;
        hash
    }

    fn get_segment(&self, hash: Integer) -> &SegmentRef<V> {
        self.segment_arr.as_ref().unwrap().get(self.get_segment_index(hash) as usize).unwrap()
    }

    fn get_segment_index(&self, hash: Integer) -> Integer {
        unsigned_right_shift!(hash, self.segment_shift, Integer) & self.segment_mask
    }
}

pub type SegmentRef<V> = Option<Arc<H2RustCell<Segment<V>>>>;

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

    /// The queue of resident cold entries <br>
    /// There is always at least one entry: the head entry.
    queue: EntryRef<V>,

    /// The queue of non-resident cold entries.
    ///
    /// There is always at least one entry: the head entry.
    queue2: EntryRef<V>,

    /// The number of times any item was moved to the top of the stack.
    stack_move_round_count: Integer,

    reentrant_mutex: ReentrantMutex<()>,

    miss_count: Integer,
    hit_count: Integer,
}

impl<V: Default + Clone + Optional> Segment<V> {
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
        let ref_mut = get_ref_mut!(self.stack);
        ref_mut.stack_prev = self.stack.clone();
        ref_mut.stack_next = self.stack.clone();

        self.queue = Entry::new_0();
        let ref_mut = get_ref_mut!(self.queue);
        ref_mut.queue_prev = self.queue.clone();
        ref_mut.queue_next = self.queue.clone();

        self.queue2 = Entry::new_0();
        let ref_mut = get_ref_mut!(self.queue2);
        ref_mut.queue_prev = self.queue2.clone();
        ref_mut.queue_next = self.queue2.clone();

        self.entries = Vec::<EntryRef<V>>::with_capacity(len as usize);
        self.entries.fill(None);
    }

    pub fn find(&self, key: Long, hash: Integer) -> EntryRef<V> {
        let index = hash & self.mask;
        // entries的大小是和mask联动的 mask=len-1 故烦心在get后unwrap
        let mut entry_ref = self.entries.get(index as usize).unwrap();
        while entry_ref.is_some() && get_ref!(entry_ref).key != key {
            entry_ref = &get_ref!(entry_ref).map_next;
        }

        entry_ref.clone()
    }

    pub fn get(&mut self, entry_ref: EntryRef<V>) -> V {
        unsafe { self.reentrant_mutex.raw().lock(); };
        // let mutex = self.reentrant_mutex.lock();

        let value = if entry_ref.is_none() {
            V::default() // 通过default()生成None
        } else {
            get_ref!(entry_ref).get_value()
        };

        // the entry was not found, or it was a non-resident entry
        if value.is_none() {
            suffix_plus_plus!(self.miss_count);
        } else {
            self.access(entry_ref);
            suffix_plus_plus!(self.hit_count);
        }

        unsafe { self.reentrant_mutex.raw().unlock(); };

        return value;
    }

    fn access(&mut self, entry_ref: EntryRef<V>) {
        let entry = get_ref!(entry_ref);
        if entry.is_hot() { // stack体系动手
            if h2_rust_cell_equals!(entry_ref, get_ref!(self.stack).stack_next) && entry.stack_next.is_some() {
                if self.stack_move_round_count - entry.top_move > self.stack_move_distance {
                    // move a hot entry to the top of the stack unless it is already there
                    let was_end = h2_rust_cell_equals!(entry_ref, get_ref!(self.stack).stack_prev);

                    self.remove_from_stack(entry_ref.clone());

                    if was_end {
                        self.prune_stack();
                    }

                    self.add_to_stack(entry_ref);
                }
            }
        } else { // queue体系动手
            let value = entry.get_value();
            if value.is_none() {
                return;
            }

            self.remove_from_queue(entry_ref.clone());

            if entry.stack_next.is_some() {
                // resident, or even non-resident (weak value reference),
                // cold entries become hot if they are on the stack
                self.remove_from_stack(entry_ref.clone());

                // which means a hot entry needs to become cold
                // (this entry is cold, that means there is at least one
                // more entry in the stack, which must be hot)
                self.convert_oldest_hot_to_cold();
            } else {
                // cold entries that are not on the stack move to the front of the queue
                self.add_to_queue(self.queue.clone(), entry_ref.clone());

                // in any case, the cold entry is moved to the top of the stack
                self.add_to_stack(entry_ref.clone());

                // but if newly promoted cold/non-resident is the only entry on a stack now
                // that means last one is cold, need to prune
                self.prune_stack();
            }
        }
    }

    /// Remove the entry from the stack. The head itself must not be removed.
    fn remove_from_stack(&mut self, entry_ref: EntryRef<V>) {
        let entry = get_ref_mut!(entry_ref);
        get_ref_mut!(entry.stack_prev).stack_next = entry.stack_next.clone();
        get_ref_mut!(entry.stack_next).stack_prev = entry.stack_prev.clone();
        entry.stack_prev = None;
        entry.stack_next = None;
        self.stack_size = self.stack_size - 1;
    }

    /// Ensure the last entry of the stack is cold.
    fn prune_stack(&mut self) {
        loop {
            let last = get_ref!(self.stack).stack_prev.clone();

            // must stop at a hot entry or the stack head,
            // but the stack head itself is also hot, so we don't have to test it
            if get_ref!(last).is_hot() {
                break;
            }

            // the cold entry is still in the queue
            self.remove_from_stack(last);
        }
    }

    fn add_to_stack(&mut self, entry_ref: EntryRef<V>) {
        let entry = get_ref_mut!(entry_ref);

        entry.stack_prev = self.stack.clone();
        entry.stack_next = get_ref!(self.stack).stack_next.clone();
        get_ref_mut!(entry.stack_next).stack_prev = entry_ref.clone();

        get_ref_mut!(self.stack).stack_next = entry_ref.clone();
        self.stack_size = self.stack_size + 1;

        entry.top_move = self.stack_move_round_count;
        self.stack_move_round_count = self.stack_move_round_count + 1;
    }

    fn remove_from_queue(&mut self, entry_ref: EntryRef<V>) {
        let entry = get_ref_mut!(entry_ref);

        get_ref_mut!(entry.queue_prev).queue_next = entry.queue_next.clone();
        get_ref_mut!(entry.queue_next).queue_prev = entry.queue_prev.clone();
        entry.queue_prev = None;
        entry.queue_next = None;

        if entry.value.is_some() {
            self.queue_size = self.queue_size - 1;
        } else {
            self.queue2size = self.queue2size - 1;
        }
    }

    fn convert_oldest_hot_to_cold(&mut self) {
        // the last entry of the stack is known to be hot
        let last = get_ref!(self.stack).stack_prev.clone();

        // never remove the stack head itself,mean the internal structure of the cache is corrupt
        if h2_rust_cell_equals!(last, self.stack) {
            panic!(" last == stack");
        }

        // remove from stack - which is done anyway in the stack pruning,but we can do it here as well
        self.remove_from_stack(last.clone());

        // adding an entry to the queue will make it cold
        self.add_to_queue(self.queue.clone(), last);

        self.prune_stack();
    }

    fn add_to_queue(&mut self, queue: EntryRef<V>, entry_ref: EntryRef<V>) {
        let entry = get_ref_mut!(entry_ref);

        entry.queue_prev = queue.clone();
        entry.queue_next = get_ref!(queue).queue_next.clone();
        get_ref_mut!(entry.queue_next).queue_prev = entry_ref.clone();
        get_ref_mut!(queue).queue_next = entry_ref.clone();

        if entry.value.is_some() {
            suffix_plus_plus!(self.queue_size);
        } else {
            suffix_plus_plus!(self.queue2size);
        }
    }
}

pub type EntryRef<V> = Option<Arc<H2RustCell<Entry<V>>>>;

#[derive(Default)]
pub struct Entry<V> {
    /// The key
    pub key: Long,

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

impl<V: Default + Clone + Optional> Entry<V> {
    pub fn new_0() -> EntryRef<V> {
        build_option_arc_h2RustCell!(Entry::default())
    }

    pub fn new_3(key: Long, value: V, memory: Integer) -> EntryRef<V> {
        let mut entry = Entry::default();
        entry.key = key;
        entry.value = value;
        entry.memory = memory;
        build_option_arc_h2RustCell!(entry)
    }

    pub fn new_1(old: &EntryRef<V>) -> EntryRef<V> {
        let mut entry = Entry::default();

        let old = get_ref!(old);
        entry.key = old.key;
        entry.value = old.value.clone();
        entry.memory = old.memory;
        build_option_arc_h2RustCell!(entry)
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