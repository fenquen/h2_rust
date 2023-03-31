use crate::h2_rust_common::{Byte, Integer};

pub type FileLockMethod = Byte;

/// This locking method means no locking is used at all.
pub const NO: FileLockMethod = 1;

/// This locking method means the cooperative file locking protocol should be used.
pub const FILE: FileLockMethod = 2;

/// This locking method means a socket is created on the given machine.
pub const SOCKET: FileLockMethod = 3;

/// Use the file system to lock the file; don't use a separate lock file
///
/// 这个和NO效果相同
pub const FS: FileLockMethod = 7;

mod test {}