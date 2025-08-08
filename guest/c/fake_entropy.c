#include <stdint.h>
#include <string.h>
#include <errno.h>

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wunused-function"
#include <immintrin.h>
#pragma clang diagnostic pop

static uint64_t get_tsc(void) {
    return __rdtsc();
}

int getentropy(void *buffer, size_t length) {
    if (buffer == NULL || length > 256) {
        errno = EINVAL;
        return -1;
    }

    uint8_t *buf = (uint8_t *)buffer;
    static uint64_t state = 0x123456789abcdef0ULL;

    while (length > 0) {
        uint64_t entropy = get_tsc();

        // Mix with evolving state
        state = state * 6364136223846793005ULL + 1442695040888963407ULL;
        entropy ^= state;

        size_t to_copy = length > 8 ? 8 : length;
        memcpy(buf, &entropy, to_copy);

        length -= to_copy;
        buf += to_copy;
    }

    return 0;
}
