use std::cmp::Ordering;
use crate::h2_rust_common::Integer;
use crate::mvstore::write_buffer::WriteBuffer;
use Ordering::{Equal, Greater, Less};
use crate::h2_rust_common::byte_buffer::ByteBuffer;
use crate::h2_rust_common::h2_rust_type::H2RustType;

pub trait DataType {
    fn compare(&self, a: &H2RustType, b: &H2RustType) -> Ordering;

    /// Perform binary search for the key within the storage
    ///
    /// @param key          to search for
    /// @param storage      to search within (an array of type T)
    /// @param size         number of data items in the storage
    /// @param initialGuess for key position
    /// @return index of the key , if found, - index of the insertion point, if not
    fn binary_search(&self, key: &H2RustType, storage: &Vec<H2RustType>, size: Integer, initial_guess: Integer) -> Integer {
        let mut low = 0;
        let mut high = size - 1;

        // the cached index minus one, so that
        // for the first time (when cachedCompare is 0),
        // the default value is used
        let mut x = initial_guess - 1;
        if x < 0 || x > high {
            x = (high as u32 >> 1) as Integer; // https://blog.csdn.net/zhizhengguan/article/details/125038005
        }

        while low <= high {
            let compare = self.compare(key, &storage[x as usize]);
            match compare {
                Greater => low = x + 1,
                Less => high = x - 1,
                Equal => return x
            }

            x = ((low + high) as u32 >> 1) as Integer;
        }

        !low
    }

    /// Calculates the amount of used memory in bytes.
    fn get_memory(&self, obj: &H2RustType) -> Integer;

    /// Whether memory estimation based on previously seen values is allowed/desirable
    fn is_memory_estimation_allowed(&self) -> bool {
        true
    }

    /// Write an object.
    ///
    /// @param buff the target buffer
    /// @param obj  the value
    fn write_2(&self, writerBuffer: &WriteBuffer, obj: &H2RustType);

    /// Write a list of objects.
    ///
    /// @param buff    the target buffer
    /// @param storage the objects
    /// @param len     the number of objects to write
    fn write_3(&self, writeBuffer: &WriteBuffer, storage: &Vec<H2RustType>, len: Integer) {
        for a in 0..len as usize {
            self.write_2(&writeBuffer, &storage[a]);
        }
    }

    /// Read an object.
    ///
    /// @param buff the source buffer
    /// @return the object
    fn read_1(&self, byteBuffer: &mut ByteBuffer) -> H2RustType;

    /// Read a list of object
    ///
    /// @param buff    the target buffer
    /// @param storage the objects
    /// @param len     the number of objects to read
    fn read_3(&self, byteBuffer: &mut ByteBuffer, storage: &mut Vec<H2RustType>, len: Integer) {
        for a in 0..len as usize {
            storage[a] = self.read_1(byteBuffer);
        }
    }

    /// Create storage object of array type to hold values
    ///
    /// @param size number of values to hold
    /// @return storage object
    fn create_storage(&self, size: Integer) -> Vec<H2RustType>;
}