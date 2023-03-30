use crate::store::file_lock_method::FileLockMethod::{FILE, FS};

pub enum FileLockMethod {
    /// This locking method means no locking is used at all.
    NO,

    /// This locking method means the cooperative file locking protocol should be used.
    FILE,

    /// This locking method means a socket is created on the given machine.
    SOCKET,

    /// Use the file system to lock the file; don't use a separate lock file.
    FS,
}

impl Default for FileLockMethod {
    fn default() -> Self {
        FS
    }
}