use anyhow::{Error, Result};
use std::fs::File;
use std::io::Error as IoError;
use crate::h2_rust_common::{Integer, Long};

extern "C" {
    fn c_lock(fd: Integer,
              is_blocking: Integer,
              position: Long,
              size: Long,
              is_shared: Integer) -> Integer;

    fn c_unlock(fd: Integer,
                position: Long,
                size: Long, ) -> Integer;
}

pub struct FileLock {
    fd: Integer,
    position: Long,
    size: Long,
}

impl FileLock {
    pub fn lock(fd: Integer,
                position: Long,
                size: Long,
                is_shared: bool) -> Result<FileLock> {
        Self::lock_internal(fd, true, position, size, is_shared)
    }

    pub fn try_lock(fd: Integer,
                    position: Long,
                    size: Long,
                    is_shared: bool) -> Result<FileLock> {
        Self::lock_internal(fd, false, position, size, is_shared)
    }

    fn lock_internal(fd: Integer,
                     blocking: bool,
                     position: Long,
                     size: Long,
                     is_shared: bool) -> Result<FileLock> {
        let ret = unsafe {
            c_lock(fd,
                   blocking as Integer,
                   position,
                   size,
                   is_shared as Integer)
        };

        match ret {
            0 => Ok(FileLock { fd, position, size }),
            _ => Err(Error::from(IoError::from_raw_os_error(ret))),
        }
    }

    pub fn release(&self) -> Result<()> {
        let ret = unsafe {
            c_unlock(self.fd, self.position, self.size)
        };

        match ret {
            0 => Ok(()),
            _ => Err(Error::from(IoError::from_raw_os_error(ret))),
        }
    }
}

mod test {
    use std::fs::{File, OpenOptions};
    use std::os::fd::AsRawFd;
    use crate::h2_rust_common::file_lock::FileLock;

    #[test]
    fn test_try_lock() {
        let mut open_options = OpenOptions::new();
        open_options.read(true);
        open_options.create(true);
        open_options.write(true);
        open_options.append(true);

        let file = open_options.open("/Users/a/Downloads/download.rs").unwrap();
        let fd = file.as_raw_fd();

        let file_lock = FileLock::try_lock(fd, 0, 0, false).unwrap();
        file_lock.release().unwrap();
    }
}