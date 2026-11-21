#pragma once
#include "pch.h"

namespace Logger {
    void Log(const wchar_t* fmt, ...);
    void SetDebugEnabled(bool enabled);
    bool IsDebugEnabled();
}
