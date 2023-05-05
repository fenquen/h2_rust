use crate::h2_rust_common::Long;
use crate::suffix_plus_plus;

#[derive(Default)]
pub struct ByteBuffer {
    data: Vec<u8>,
    position: usize,
    capacity: usize,
    limit: usize,
}

pub fn allocate(size: usize) -> ByteBuffer {
    let data = Vec::with_capacity(size);

    wrapVec(data)
}

pub fn wrapSlice(slice: &[u8]) -> ByteBuffer {
    let mut byteBuffer = allocate(slice.len());

    for v in slice {
        byteBuffer.data[suffix_plus_plus!(byteBuffer.position)] = *v;
    }

    byteBuffer
}

pub fn wrapVec(vec: Vec<u8>) -> ByteBuffer {
    ByteBuffer {
        data: vec,
        position: 0,
        capacity: vec.capacity(),
        limit: vec.capacity(),
    }
}

impl ByteBuffer {
    pub fn getCapacity(&self) -> usize {
        self.capacity
    }

    pub fn getRemaining(&self) -> usize {
        self.limit - self.position
    }

    pub fn hasRemaining(&self) -> bool {
        self.limit > self.position
    }

    pub fn getPosition(&self) -> usize {
        self.position
    }

    pub fn getLimit(&self) -> usize {
        self.limit
    }

    pub fn setPosition(&mut self, newPosition: usize) {
        if newPosition > self.limit {
            panic!("newPosition > limit")
        }

        self.position = newPosition;
    }

    pub fn setLimit(&mut self, newLimit: usize) {
        if newLimit > self.capacity {
            panic!("newLimit > capacity")
        }

        if self.position > newLimit {
            panic!("position > newLimit")
        }

        self.limit = newLimit;
    }

    pub fn getByte(&mut self) -> u8 {
        let oldPosition = self.position;
        self.setPosition(oldPosition + 1);
        self.data[oldPosition]
    }

    pub fn putByte(&mut self, b: u8) {
        let oldPosition = self.position;
        self.setPosition(self.position + 1);
        self.data[oldPosition] = b
    }

    pub fn getU16(&mut self) -> u16 {
        self.setPosition(self.position + 2);
        (self.getByte() as u16) << 8 | self.getByte() as u16
    }

    pub fn putU16(&mut self, u: u16) {
        let oldPosition = self.position;

        self.setPosition(self.position + 2);

        self.data[oldPosition] = (u >> 8) as u8;
        self.data[oldPosition + 1] = u as u8;
    }

    pub fn putI32(&mut self, a: i32) {
        let oldPosition = self.position;

        self.setPosition(self.position + 4);

        self.data[oldPosition] = (a >> 24) as u8;
        self.data[oldPosition + 1] = (a >> 16) as u8;
        self.data[oldPosition + 2] = (a >> 8) as u8;
        self.data[oldPosition + 3] = a as u8;
    }

    pub fn putSlice(&mut self, slice: &[u8]) {
        self.putSlice_(slice, 0, slice.len());
    }

    pub fn putSlice_(&mut self, slice: &[u8], offset: usize, length: usize) {
        if offset + length > slice.len() {
            panic!("offset + length > len(slice)")
        }

        if length > self.getRemaining() {
            panic!("byteBuffer剩下的空间已放不下该slice")
        }

        let oldPosition = self.position;

        self.setPosition(self.position + length);

        let mut oldPosition = oldPosition;
        for d in 0..length {
            self.data[suffix_plus_plus!(oldPosition)] = slice[offset + d];
        }
    }

    pub fn getSlice(&mut self, dest: &mut [u8]) {
        if dest.len() > self.getRemaining() {
            panic!("len(dest) > byteBuffer.getRemaining()")
        }

        for a in 0..dest.len() {
            dest[a] = self.getByte();
        }
    }

    pub fn putByteBuffer(&mut self, src: &mut ByteBuffer) {
        if self as *const ByteBuffer == src as *const ByteBuffer {
            panic!("self == src")
        }

        if src.getRemaining() > self.getRemaining() {
            panic!("src.getRemaining() > self.getRemaining()")
        }

        loop {
            // 两者由一个已没有空间了
            if !(src.hasRemaining() && self.hasRemaining()) {
                return;
            }

            self.putByte(src.getByte())
        }
    }

    pub fn getInternalSlice(&self) -> &[u8] {
        &self.data
    }

    pub fn clear(&mut self) {
        self.position = 0;
        self.limit = self.capacity
    }

    pub fn flip(&mut self) {
        self.setLimit(self.position);
        self.position = 0;
    }

    pub fn extract(&self) -> &[u8] {
        return &self.data[self.position..self.limit];
    }

    pub fn extractWithPosLen(&self, position: usize, len: usize) -> &[u8] {
        return &self.data[position..position + len];
    }

    pub fn skip(&mut self, count: usize) {
        self.setPosition(self.position + count);
    }
}

impl From<&[u8]> for ByteBuffer {
    fn from(slice: &[u8]) -> Self {
        wrapSlice(slice)
    }
}

impl From<Vec<u8>> for ByteBuffer {
    fn from(vec: Vec<u8>) -> Self {
        wrapVec(vec)
    }
}