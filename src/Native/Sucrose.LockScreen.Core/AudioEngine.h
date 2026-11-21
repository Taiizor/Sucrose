#pragma once
#include "pch.h"

namespace AudioEngine {
    bool Start(const std::wstring& mediaPath, int volume);
    void Stop();
}
