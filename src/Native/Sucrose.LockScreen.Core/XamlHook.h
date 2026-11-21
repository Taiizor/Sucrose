#pragma once
#include "pch.h"

namespace XamlHook {
    bool Start();
    void Stop();
    void TeardownOnUIThread();
    void TripFailSafe(const wchar_t* reason);
}
