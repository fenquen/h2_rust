use bytebuffer::ByteBuffer;
use crate::h2_rust_common::Integer;
use crate::mvstore::write_buffer::WriteBuffer;


pub trait DataType<T> {
    /// Perform binary search for the key within the storage
    ///
    /// @param key          to search for
    /// @param storage      to search within (an array of type T)
    /// @param size         number of data items in the storage
    /// @param initialGuess for key position
    /// @return index of the key , if found, - index of the insertion point, if not
    fn binary_search(&self, key: T, storage: &Vec<T>, size: Integer, initial_guess: Integer) -> Integer;

    /// Calculates the amount of used memory in bytes.
    fn get_memory(&self, obj: T) -> Integer;

    /// Whether memory estimation based on previously seen values is allowed/desirable
    fn is_memory_estimation_allowed(&self) -> bool;

    /// Write an object.
    ///
    /// @param buff the target buffer
    /// @param obj  the value
    fn write_2(&self, buff: WriteBuffer, obj: T);

    /// Write a list of objects.
    ///
    /// @param buff    the target buffer
    /// @param storage the objects
    /// @param len     the number of objects to write
    fn write_3(&self, buff: WriteBuffer, storage: Vec<T>, len: Integer);

    /// Read an object.
    ///
    /// @param buff the source buffer
    /// @return the object
    fn read_1(&self, buff: ByteBuffer) -> T;

    /// Read a list of objects.
    ///
    /// @param buff    the target buffer
    /// @param storage the objects
    /// @param len     the number of objects to read
    fn read_3(&self, buff: ByteBuffer, storage: Vec<T>, len: Integer);

    /// Create storage object of array type to hold values
    ///
    /// @param size number of values to hold
    /// @return storage object
    fn create_storage(&self, size: Integer) -> Vec<T>;
}