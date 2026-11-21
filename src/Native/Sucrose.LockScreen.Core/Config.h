#pragma once
#include "pch.h"
#include "Common.h"

struct LockScreenConfig {
    // General
    bool enabled = true;
    std::wstring mediaPath = L"";
    FitMode fitMode = kFitCover;
    int speed = 100;
    int audioVolume = 0;
    int folderOrder = 0; // 0: random, 1: sequential, 2: fixed

    // Performance & Power
    int renderWidth = 960;
    bool hwDecode = true;
    bool playOnPasswordScreen = true;
    DWORD idlePauseSeconds = 300;
    int lowBatteryThreshold = 20;
    int lowBatteryLevel = 2; // 0: off, 1: light, 2: balanced, 3: maximum
    bool lowBatteryOnAC = false;

    // Screens & Boot
    bool showOnShutdown = false;
    bool bootStillFrame = true;
    DWORD bootStillSeconds = 180;

    // Color Adjustments
    int hue = 0;
    int saturation = 100;
    int brightness = 0;
    int contrast = 100;
    int gamma = 100;
    int dim = 0;

    // Precalculated color tables
    bool isIdentity = true;
    bool isLutOnly = false;
    int colorMatrix[9] = {};
    uint8_t lut[256] = {};

    void CalculateColorMatrices();
    void ApplyColor(uint32_t* pixels, size_t count) const;
};

namespace ConfigManager {
    LockScreenConfig& Current();
    bool Load(const std::wstring& configFilePath);
    std::wstring ResolveMediaFile(const std::wstring& path, int folderOrder);
}
