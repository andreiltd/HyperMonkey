#include <time.h>

#include "mozilla/TimeStamp.h"

// stub for now
namespace mozilla {

double BaseTimeDurationPlatformUtils::ToSeconds(int64_t aTicks) {
  return double(aTicks) / kNsPerSecd;
}

double BaseTimeDurationPlatformUtils::ToSecondsSigDigits(int64_t aTicks) {
  // don't report a value < mResolution ...
  int64_t valueSigDigs = sResolution * (aTicks / sResolution);
  // and chop off insignificant digits
  valueSigDigs = sResolutionSigDigs * (valueSigDigs / sResolutionSigDigs);
  return double(valueSigDigs) / kNsPerSecd;
}

int64_t BaseTimeDurationPlatformUtils::TicksFromMilliseconds(
    double aMilliseconds) {
  double result = aMilliseconds * kNsPerMsd;
  if (result > double(INT64_MAX)) {
    return INT64_MAX;
  }
  if (result < INT64_MIN) {
    return INT64_MIN;
  }

  return result;
}

int64_t BaseTimeDurationPlatformUtils::ResolutionInTicks() {
  return static_cast<int64_t>(sResolution);
}

double BaseTimeDurationPlatformUtils::ToSeconds(int64_t aTicks) {
  return double(aTicks) / kNsPerSecd;
}

void TimeStamp::Startup() { }

void TimeStamp::Shutdown() { }

TimeStamp TimeStamp::Now(bool aHighResolution) {
    return TimeStamp(0);
}

uint64_t TimeStamp::ComputeProcessUptime() {
    return 0;
}

} // namespace mozilla
