#include <errno.h>
#include <fcntl.h>
#include <stdint.h>

int c_lock(int fd, int is_blocking, int64_t position, int64_t size, int is_shared) {
    if (fd < 0) {
        return EBADF;
    }

    struct flock flock;
    flock.l_type = is_shared ? F_RDLCK : F_WRLCK;
    flock.l_whence = SEEK_SET;
    flock.l_start = position;
    flock.l_len = size;

    if (fcntl(fd, is_blocking ? F_SETLKW : F_SETLK, &flock) == -1) {
        return errno;
    }

    return 0;
}

int c_unlock(int fd, int64_t position, int64_t size) {
    if (fd < 0) {
        return EBADF;
    }

    struct flock flock;
    flock.l_type = F_UNLCK;
    flock.l_whence = SEEK_SET;
    flock.l_start = position;
    flock.l_len = size;

    if (fcntl(fd, F_SETLK, &flock) == -1) {
        return errno;
    }

    return 0;
}
