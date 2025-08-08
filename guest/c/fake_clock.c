#include <sys/time.h>
#include <time.h>
#include <errno.h>
#include <stddef.h>

static const unsigned long FAKE_TIME_START_DEFAULT = 3600UL * 1000000UL;
static const unsigned long FAKE_TIME_INCREMENT = 1000UL;

static volatile unsigned long fake_time_counter = 0;
static volatile int time_initialized = 0;

static void ensure_time_initialized(void) {
    if (__builtin_expect(!time_initialized, 0)) {
        if (__sync_bool_compare_and_swap(&time_initialized, 0, 1)) {
            fake_time_counter = FAKE_TIME_START_DEFAULT;
            __sync_synchronize();
        }
    }
}

static unsigned long atomic_advance_time(void) {
    return __sync_fetch_and_add(&fake_time_counter, FAKE_TIME_INCREMENT);
}

int gettimeofday(struct timeval *__restrict tv, [[maybe_unused]] void *__restrict __tz) {
    ensure_time_initialized();

    unsigned long current_time = atomic_advance_time();
    tv->tv_sec = 1609459200 + (current_time / 1000000);
    tv->tv_usec = (current_time % 1000000);

    return 0;
}

int clock_gettime(clockid_t clk_id, struct timespec *tp) {
    ensure_time_initialized();

    unsigned long current_time = atomic_advance_time();

    switch (clk_id) {
        case CLOCK_REALTIME:
            tp->tv_sec = 1609459200 + (current_time / 1000000);
            tp->tv_nsec = (current_time % 1000000) * 1000;
            break;
        case CLOCK_MONOTONIC:
            tp->tv_sec = current_time / 1000000;
            tp->tv_nsec = (current_time % 1000000) * 1000;
            break;
        default:
            errno = EINVAL;
            return -1;
    }

    return 0;
}

int clock_getres(clockid_t clk_id, struct timespec *res) {
    if (res == NULL) {
        errno = EFAULT;
        return -1;
    }

    switch (clk_id) {
        case CLOCK_REALTIME:
        case CLOCK_MONOTONIC:
            res->tv_sec = 0;
            res->tv_nsec = 1000;
            break;

        default:
            errno = EINVAL;
            return -1;
    }

    return 0;
}
