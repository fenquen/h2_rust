use std::cmp;
use std::env::var;
use std::ops::Deref;
use std::sync::Arc;
use usync::lock_api::RawMutex;
use usync::ReentrantMutex;
use crate::{build_option_arc_h2RustCell, db_error_template, get_ref, get_ref_mut, h2_rust_cell_equals, suffix_minus_minus, suffix_plus_plus, throw, unsigned_right_shift};
use crate::api::error_code;
use crate::h2_rust_common::{Integer, Long, MyMutex, Nullable, Optional, ULong};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::data_utils;
use crate::mvstore::page::{get, Page};
use crate::h2_rust_common::UInteger;
use crate::message::db_error::DbError;
use anyhow::Result;

#[derive(Default)]
pub struct CacheLongKeyLIRS<V> {
    /// the maximum memory this cache should use.
    max_memory: Long,
    segmentArr: Option<Vec<SegmentRef<V>>>,
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

        self.setMaxMemory(config.max_memory);
        self.non_resident_queue_size = config.non_resident_queue_size;
        self.non_resident_queue_size_high = config.non_resident_queue_size_high;

        self.segment_mask = self.segment_count - 1;
        self.stack_move_distance = config.stack_move_distance;
        self.segmentArr = Some(Vec::<SegmentRef<V>>::with_capacity(self.segment_count as usize));

        self.clear();

        // use the high bits for the segment
        self.segment_shift = (32 - Integer::count_ones(self.segment_mask)) as Integer;
    }

    pub fn setMaxMemory(&mut self, max_memory: Long) {
        data_utils::check_argument(max_memory > 0, "Max memory must be larger than 0");
        self.max_memory = max_memory;

        if self.segmentArr.is_some() {
            let segment_arr = self.segmentArr.as_mut().unwrap();
            let max = 1 + max_memory / segment_arr.len() as Long;
            for segment_ref in segment_arr {
                get_ref_mut!(segment_ref).maxMemory = max;
            }
        }
    }

    pub fn clear(&mut self) {
        let max = self.get_max_item_size();
        let segment_arr = self.segmentArr.as_mut().unwrap();
        for _ in 0..self.segment_count {
            segment_arr.push(build_option_arc_h2RustCell!(Segment::<V>::new5(
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


    pub fn put(&mut self, key: Long, value: V, memory: Integer) -> Result<()> {
        if value.is_none() {
            throw!(DbError::get(error_code::GENERAL_ERROR_1,vec!["The value may not be null"]));
        }

        let hash = getHash(key);
        let segmentIndex = self.getSegmentIndex(hash);
        let segmentRef = self.segmentArr.as_ref().unwrap().get(segmentIndex as usize).unwrap();

        // check whether resize is required: synchronize on s, to avoid
        // concurrent resizes (concurrent reads read from the old segment)
        let segmentClone = segmentRef.clone();
        let clone = get_ref!(segmentClone).reentrantMutexPtr.clone();
        let mutexGuard = get_ref!(clone).lock();

        let segmentRef = self.resizeIfNeeded(segmentRef.clone(), segmentIndex as usize);
        get_ref_mut!(segmentRef).put(key, hash, value, memory);

        Ok(())
    }

    fn resizeIfNeeded(&mut self, mut segmentRef: SegmentRef<V>, segmentIndex: usize) -> SegmentRef<V> {
        let newLen = get_ref!(segmentRef).getNewMapLen();
        if newLen == 0 {
            return segmentRef;
        }

        // another thread might have resized (as we retrieved the segment before synchronizing on it)
        let s2 = self.segmentArr.as_ref().unwrap().get(segmentIndex);
        if segmentRef.as_ref().unwrap().equals(s2.unwrap().as_ref().unwrap()) {
            // no other thread resized, so we do
            segmentRef = Segment::<V>::new2(segmentRef.clone(), newLen);
            self.segmentArr.as_mut().unwrap().insert(segmentIndex, segmentRef.clone());
        }

        return segmentRef;
    }

    pub fn get(&mut self, key: Long) -> V {
        let hash = getHash(key);
        let segment_ref = self.get_segment(hash);
        let entry_ref = get_ref!(segment_ref).find(key, hash);
        get_ref_mut!(segment_ref).get(entry_ref) // 因为该函数内部需要V上有Optional相应函数使得CacheLongKeyLIRS的V也要实现Optional,部下污染了上头
    }

    fn get_segment(&self, hash: Integer) -> &SegmentRef<V> {
        self.segmentArr.as_ref().unwrap().get(self.getSegmentIndex(hash) as usize).unwrap()
    }

    fn getSegmentIndex(&self, hash: Integer) -> Integer {
        unsigned_right_shift!(hash, self.segment_shift, Integer) & self.segment_mask
    }
}

fn getHash(key: Long) -> Integer {
    let mut hash = ((key as ULong >> 32) as Long ^ key) as Integer;
    // a supplemental secondary hash function to protect against hash codes that don't differ much
    hash = unsigned_right_shift!(hash, 16, Integer);
    hash = (unsigned_right_shift!(hash, 16, Integer) ^ hash) * 0x45d9f3b;
    hash = (unsigned_right_shift!(hash, 16, Integer) ^ hash) * 0x45d9f3b;
    hash = unsigned_right_shift!(hash, 16, Integer) ^ hash;
    hash
}

pub type SegmentRef<V> = Option<Arc<H2RustCell<Segment<V>>>>;

#[derive(Default)]
pub struct Segment<V> {
    /// The number of (hot, cold, and non-resident) entries in the map.
    mapSize: Integer,

    /// The size of the LIRS queue for resident cold entries.
    queue_size: Integer,

    /// The size of the LIRS queue for non-resident cold entries.
    queue2size: Integer,

    /// The number of cache hits.
    hits: Long,

    /// The number of cache misses.
    misses: Long,

    /// The map array. The size is always a power of 2.
    entries: Vec<EntrySharedPtr<V>>,

    /// The currently used memory.
    usedMemory: Long,

    /// How many other item are to be moved to the top of the stack before the current item is moved.
    stackMoveDistance: Integer,

    /// Set the maximum memory this cache should use. This will not
    /// immediately cause entries to get removed however; it will only change
    /// the limit. To resize the internal array, call the clear method.
    /// the maximum size (1 or larger) this cache should use in bytes
    maxMemory: Long,

    /// The bit mask that is applied to the key hash code to get the index in
    /// the map array. The mask is the length of the array minus one.
    mask: Integer,

    /// Low watermark for the number of entries in the non-resident queue,
    /// as a factor of the number of entries in the map.
    nonResidentQueueSize: Integer,

    /// High watermark for the number of entries in the non-resident queue, as a factor of the number of entries in the map.
    nonResidentQueueSizeHigh: Integer,

    /// The stack of recently referenced elements. This includes all hot
    /// entries, and the recently referenced cold entries. Resident cold
    /// entries that were not recently referenced, as well as non-resident cold entries, are not in the stack.
    ///
    /// There is always at least one entry: the head entry.
    stack: EntrySharedPtr<V>,

    /// The number of entries in the stack.
    stackSize: Integer,

    /// The queue of resident cold entries <br>
    /// There is always at least one entry: the head entry.
    queue: EntrySharedPtr<V>,

    /// The queue of non-resident cold entries.
    /// There is always at least one entry: the head entry.
    queue2: EntrySharedPtr<V>,

    /// The number of times any item was moved to the top of the stack.
    stackMoveRoundCount: Integer,

    // reentrantMutex: ReentrantMutex<()>,
    reentrantMutexPtr: Option<Arc<H2RustCell<ReentrantMutex<()>>>>,

    missCount: Integer,
    hitCount: Integer,
}

impl<V: Default + Clone + Optional> Segment<V> {
    pub fn default1() -> Segment<V> {
        let mut segment: Segment<V> = Default::default();
        segment.reentrantMutexPtr = build_option_arc_h2RustCell!(ReentrantMutex::<()>::default());
        segment
    }

    pub fn new2(old: SegmentRef<V>, len: Integer) -> SegmentRef<V> {
        let old = get_ref!(old);

        let mut segment = Segment::<V>::default1();

        Self::init5(&mut segment, old.maxMemory,
                    old.stackMoveDistance,
                    len,
                    old.nonResidentQueueSize,
                    old.nonResidentQueueSizeHigh);

        segment.hitCount = old.hitCount;
        segment.missCount = old.missCount;

        let mut entrySharedPtr = get_ref!(old.stack).stackPrev.clone();
        while !h2_rust_cell_equals!(entrySharedPtr , old.stack) {
            let e = Entry::<V>::new1(&entrySharedPtr);
            segment.addToMap(e.clone());
            segment.addToStack(e);
            entrySharedPtr = get_ref!(entrySharedPtr).stackPrev.clone();
        }

        entrySharedPtr = get_ref!(old.queue).queuePrev.clone();
        while !h2_rust_cell_equals!(entrySharedPtr, old.queue) {
            let mut e = segment.find(get_ref!(entrySharedPtr).key, getHash(get_ref!(entrySharedPtr).key));
            if e.is_none() {
                e = Entry::<V>::new1(&entrySharedPtr);
                segment.addToMap(e.clone());
            }
            segment.addToQueue(segment.queue.clone(), e);
            entrySharedPtr = get_ref!(entrySharedPtr).queuePrev.clone();
        }

        entrySharedPtr = get_ref!(old.queue2).queuePrev.clone();
        while !h2_rust_cell_equals!(entrySharedPtr, old.queue2) {
            let mut e = segment.find(get_ref!(entrySharedPtr).key, getHash(get_ref!(entrySharedPtr).key));
            if e.is_none() {
                e = Entry::<V>::new1(&entrySharedPtr);
                segment.addToMap(e.clone());
            }
            segment.addToQueue(segment.queue2.clone(), e);
            entrySharedPtr = get_ref!(entrySharedPtr).queuePrev.clone();
        }

        build_option_arc_h2RustCell!(segment)
    }

    pub fn new5(max_memory: Long,
                stackMoveDistance: Integer,
                len: Integer,
                nonResidentQueueSize: Integer,
                nonResidentQueueSizeHigh: Integer) -> Segment<V> {
        let mut segment = Segment::<V>::default1();
        Self::init5(&mut segment,
                    max_memory,
                    stackMoveDistance,
                    len,
                    nonResidentQueueSize,
                    nonResidentQueueSizeHigh);
        segment
    }

    fn init5(&mut self,
             max_memory: Long,
             stack_move_distance: Integer,
             len: Integer,
             non_resident_queue_size: Integer,
             non_resident_queue_size_high: Integer) {
        self.maxMemory = max_memory;
        self.stackMoveDistance = stack_move_distance;
        self.nonResidentQueueSize = non_resident_queue_size;
        self.nonResidentQueueSizeHigh = non_resident_queue_size_high;

        // the bit mask has all bits set
        self.mask = len - 1;

        // initialize the stack and queue heads
        self.stack = Entry::new0();
        let ref_mut = get_ref_mut!(self.stack);
        ref_mut.stackPrev = self.stack.clone();
        ref_mut.stackNext = self.stack.clone();

        self.queue = Entry::new0();
        let ref_mut = get_ref_mut!(self.queue);
        ref_mut.queuePrev = self.queue.clone();
        ref_mut.queueNext = self.queue.clone();

        self.queue2 = Entry::new0();
        let ref_mut = get_ref_mut!(self.queue2);
        ref_mut.queuePrev = self.queue2.clone();
        ref_mut.queueNext = self.queue2.clone();

        self.entries = Vec::<EntrySharedPtr<V>>::with_capacity(len as usize);
        self.entries.fill(None);
    }

    pub fn find(&self, key: Long, hash: Integer) -> EntrySharedPtr<V> {
        let index = hash & self.mask;
        // entries的大小是和mask联动的 mask=len-1 故烦心在get后unwrap
        let mut entrySharedPtr = self.entries.get(index as usize).unwrap();
        while entrySharedPtr.is_some() && get_ref!(entrySharedPtr).key != key {
            entrySharedPtr = &get_ref!(entrySharedPtr).mapNext;
        }

        entrySharedPtr.clone()
    }

    pub fn get(&mut self, entry_ref: EntrySharedPtr<V>) -> V {
        let clone = self.reentrantMutexPtr.clone();
        let mutexGuard = get_ref!(clone).lock();

        // let mutex = self.reentrant_mutex.lock();

        let value = if entry_ref.is_none() {
            V::default() // 通过default()生成None
        } else {
            get_ref!(entry_ref).getValue()
        };

        // the entry was not found, or it was a non-resident entry
        if value.is_none() {
            suffix_plus_plus!(self.missCount);
        } else {
            self.access(entry_ref);
            suffix_plus_plus!(self.hitCount);
        }

        return value;
    }

    fn access(&mut self, entry_ref: EntrySharedPtr<V>) {
        let entry = get_ref!(entry_ref);
        if entry.isHot() { // stack体系动手
            if h2_rust_cell_equals!(entry_ref, get_ref!(self.stack).stackNext) && entry.stackNext.is_some() {
                if self.stackMoveRoundCount - entry.topMove > self.stackMoveDistance {
                    // move a hot entry to the top of the stack unless it is already there
                    let was_end = h2_rust_cell_equals!(entry_ref, get_ref!(self.stack).stackPrev);

                    self.removeFromStack(entry_ref.clone());

                    if was_end {
                        self.pruneStack();
                    }

                    self.addToStack(entry_ref);
                }
            }
        } else { // queue体系动手
            let value = entry.getValue();
            if value.is_none() {
                return;
            }

            self.removeFromQueue(entry_ref.clone());

            if entry.stackNext.is_some() {
                // resident, or even non-resident (weak value reference),
                // cold entries become hot if they are on the stack
                self.removeFromStack(entry_ref.clone());

                // which means a hot entry needs to become cold
                // (this entry is cold, that means there is at least one
                // more entry in the stack, which must be hot)
                self.convertOldestHotToCold();
            } else {
                // cold entries that are not on the stack move to the front of the queue
                self.addToQueue(self.queue.clone(), entry_ref.clone());

                // in any case, the cold entry is moved to the top of the stack
                self.addToStack(entry_ref.clone());

                // but if newly promoted cold/non-resident is the only entry on a stack now
                // that means last one is cold, need to prune
                self.pruneStack();
            }
        }
    }

    fn put(&mut self, key: Long, hash: Integer, value: V, memory: Integer) -> V {
        let clone = self.reentrantMutexPtr.clone();
        let mutexGuard = get_ref!(clone).lock();

        let mut entry = self.find(key, hash);
        let existed = entry.is_none();

        let mut old: V = Default::default();
        assert!(old.is_none());

        if existed {
            old = get_ref!(entry).getValue();
            self.remove(key, hash);
        }

        // the new entry is too big to fit
        if memory as Long > self.maxMemory {
            return old;
        }

        entry = Entry::<V>::new3(key, value, memory);

        let index = (hash & self.mask) as usize;

        get_ref_mut!(entry).mapNext = self.entries[index].clone();
        self.entries[index] = entry.clone();

        self.usedMemory += memory;
        if self.usedMemory > self.maxMemory {
            // old entries needs to be removed
            evict();
        }

        old
    }

    fn remove(&mut self, key: Long, hash: Integer) -> V {
        let clone = self.reentrantMutexPtr.clone();
        let mutexGuard = get_ref!(clone).lock();

        let index = (hash & self.mask) as usize;

        let mut entry = self.entries[index].clone();
        if entry.is_none() {
            return V::default();
        }

        if get_ref!(entry).key == key {
            self.entries[index] = get_ref!(entry).mapNext.clone();
        } else {
            let mut last: EntrySharedPtr<V>;
            loop {
                last = entry.clone();
                entry = get_ref!(entry).mapNext.clone();
                if entry.is_none() {
                    return V::default();
                }

                if get_ref!(entry).key == key {
                    break;
                }
            }

            get_ref_mut!(last).mapNext = get_ref!(entry).mapNext.clone();
        }

        let old = get_ref!(entry).getValue();

        suffix_minus_minus!(self.mapSize);
        self.usedMemory -= get_ref!(entry).getMemory() as Long;

        if get_ref!(entry).stackNext.is_some() {
            self.removeFromStack(entry.clone());
        }

        if get_ref!(entry).isHot() {
            entry = get_ref!(self.queue).queueNext.clone();

            if !h2_rust_cell_equals!(entry, self.queue) {
                self.removeFromQueue(entry.clone());

                if get_ref!(entry).stackNext.is_none() {
                    self.addToStackBottom(entry.clone());
                }
            }

            self.pruneStack();
        } else {
            self.removeFromQueue(entry.clone());
        }

        old
    }


    /// evict cold entries (resident and non-resident) until the memory limit
    /// is reached. The new entry is added as a cold entry, except if it is the only entry.
    fn evict(&mut self) {
        loop {
            evictBlock();

            if usedMemory > maxMemory{

            }
        }
    }

    /// Remove the entry from the stack. The head itself must not be removed.
    fn removeFromStack(&mut self, entry_ref: EntrySharedPtr<V>) {
        let entry = get_ref_mut!(entry_ref);
        get_ref_mut!(entry.stackPrev).stackNext = entry.stackNext.clone();
        get_ref_mut!(entry.stackNext).stackPrev = entry.stackPrev.clone();
        entry.stackPrev = None;
        entry.stackNext = None;
        self.stackSize = self.stackSize - 1;
    }

    /// Ensure the last entry of the stack is cold.
    fn pruneStack(&mut self) {
        loop {
            let last = get_ref!(self.stack).stackPrev.clone();

            // must stop at a hot entry or the stack head,
            // but the stack head itself is also hot, so we don't have to test it
            if get_ref!(last).isHot() {
                break;
            }

            // the cold entry is still in the queue
            self.removeFromStack(last);
        }
    }

    fn addToStack(&mut self, entry: EntrySharedPtr<V>) {
        let entryMutRef = get_ref_mut!(entry);

        entryMutRef.stackPrev = self.stack.clone();
        entryMutRef.stackNext = get_ref!(self.stack).stackNext.clone();
        get_ref_mut!(entryMutRef.stackNext).stackPrev = entry.clone();

        get_ref_mut!(self.stack).stackNext = entry.clone();
        self.stackSize = self.stackSize + 1;

        entryMutRef.topMove = self.stackMoveRoundCount;
        self.stackMoveRoundCount = self.stackMoveRoundCount + 1;
    }

    fn addToStackBottom(&mut self, entry: EntrySharedPtr<V>) {
        let entryMutRef = get_ref_mut!(entry);

        entryMutRef.stackNext = self.stack.clone();
        entryMutRef.stackPrev = get_ref!(self.stack).stackPrev.clone();
        get_ref_mut!(entryMutRef.stackPrev).stackNext = entry.clone();
        get_ref_mut!(self.stack).stackPrev = entry.clone();
        suffix_plus_plus!(self.stackSize);
    }

    fn removeFromQueue(&mut self, entry_ref: EntrySharedPtr<V>) {
        let entry = get_ref_mut!(entry_ref);

        get_ref_mut!(entry.queuePrev).queueNext = entry.queueNext.clone();
        get_ref_mut!(entry.queueNext).queuePrev = entry.queuePrev.clone();
        entry.queuePrev = None;
        entry.queueNext = None;

        if entry.value.is_some() {
            self.queue_size = self.queue_size - 1;
        } else {
            self.queue2size = self.queue2size - 1;
        }
    }

    fn convertOldestHotToCold(&mut self) {
        // the last entry of the stack is known to be hot
        let last = get_ref!(self.stack).stackPrev.clone();

        // never remove the stack head itself,mean the internal structure of the cache is corrupt
        if h2_rust_cell_equals!(last, self.stack) {
            panic!(" last == stack");
        }

        // remove from stack - which is done anyway in the stack pruning,but we can do it here as well
        self.removeFromStack(last.clone());

        // adding an entry to the queue will make it cold
        self.addToQueue(self.queue.clone(), last);

        self.pruneStack();
    }

    fn addToQueue(&mut self, queue: EntrySharedPtr<V>, entry_ref: EntrySharedPtr<V>) {
        let entry = get_ref_mut!(entry_ref);

        entry.queuePrev = queue.clone();
        entry.queueNext = get_ref!(queue).queueNext.clone();
        get_ref_mut!(entry.queueNext).queuePrev = entry_ref.clone();
        get_ref_mut!(queue).queueNext = entry_ref.clone();

        if entry.value.is_some() {
            suffix_plus_plus!(self.queue_size);
        } else {
            suffix_plus_plus!(self.queue2size);
        }
    }

    /// Calculate the new number of hash table buckets if the internal map should be re-sized.
    ///
    /// return 0 if no resizing is needed, or the new length
    pub fn getNewMapLen(&self) -> Integer {
        let len = self.mask + 1;

        // more than 75% usage
        if len * 3 < self.mapSize * 4 && len < (1 << 28) {
            return len * 2;
        }

        if len > 32 && len / 8 > self.mapSize {
            // less than 12% usage
            return len / 2;
        }

        0
    }

    fn addToMap(&mut self, entrySharedPtr: EntrySharedPtr<V>) {
        let entryMutRef = get_ref_mut!(entrySharedPtr);

        let index = (getHash(entryMutRef.key) & self.mask) as usize;
        entryMutRef.mapNext = self.entries[index].clone();
        self.entries[index] = entrySharedPtr.clone();
        self.usedMemory += entryMutRef.getMemory() as Long;
        suffix_plus_plus!(self.mapSize);
    }
}

pub type EntrySharedPtr<V> = Option<Arc<H2RustCell<Entry<V>>>>;

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
    topMove: Integer,

    /// The next entry in the stack.
    stackNext: EntrySharedPtr<V>,

    /// The previous entry in the stack.
    stackPrev: EntrySharedPtr<V>,

    /// The next entry in the queue (either the resident queue or the non-resident queue).
    queueNext: EntrySharedPtr<V>,

    /// The previous entry in the queue.
    queuePrev: EntrySharedPtr<V>,

    /// The next entry in the map (the chained entry).
    mapNext: EntrySharedPtr<V>,
}

impl<V: Default + Clone + Optional> Entry<V> {
    pub fn new0() -> EntrySharedPtr<V> {
        build_option_arc_h2RustCell!(Entry::default())
    }

    pub fn new3(key: Long, value: V, memory: Integer) -> EntrySharedPtr<V> {
        let mut entry = Entry::default();
        entry.key = key;
        entry.value = value;
        entry.memory = memory;
        build_option_arc_h2RustCell!(entry)
    }

    pub fn new1(old: &EntrySharedPtr<V>) -> EntrySharedPtr<V> {
        let mut entry = Entry::default();

        let old = get_ref!(old);

        entry.key = old.key;
        entry.value = old.value.clone();
        entry.memory = old.memory;

        entry.topMove = old.topMove.clone();

        build_option_arc_h2RustCell!(entry)
    }

    /// whether this entry is hot. Cold entries are in one of the two queues.
    pub fn isHot(&self) -> bool {
        self.queueNext.is_none()
    }

    pub fn getValue(&self) -> V {
        self.value.clone()
    }

    pub fn getMemory(&self) -> Integer {
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