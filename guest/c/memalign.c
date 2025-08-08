#include <errno.h>
#include <stdint.h>
#include <stdlib.h>

extern void *aligned_alloc(size_t alignment, size_t size);

int posix_memalign(void **memptr, size_t alignment, size_t size) {
    if (alignment < sizeof(void *) || (alignment & (alignment - 1)) != 0) {
        return EINVAL;
    }

    void *ptr = aligned_alloc(alignment, size);

    if (ptr == NULL) {
        return ENOMEM;
    }

    *memptr = ptr;
    return 0;
}

void *memalign (size_t alignment, size_t size) {
    if (alignment < sizeof(void *) ||
        (alignment & (alignment - 1)) != 0) {
        errno = EINVAL;
        return NULL;
    }

    if ((size % alignment) != 0) {
        size += alignment - (size % alignment);
    }

    return aligned_alloc(alignment, size);
}
