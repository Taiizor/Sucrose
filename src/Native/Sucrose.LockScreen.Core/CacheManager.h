#pragma once
#include "pch.h"
#include "Common.h"

namespace CacheManager {
    std::wstring GetCacheFilePath(const std::wstring& mediaPath, int width, int height, FitMode fitMode);
    bool LoadFirstFrame(const std::wstring& cachePath);
    void SaveFirstFrame(const std::wstring& cachePath, const uint8_t* rawPixels);

    bool LoadLastFrame();
    void SaveLastFrame();

    void PruneCache();
}
