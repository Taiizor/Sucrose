#include "pch.h"
#include "Common.h"

namespace CoreState {
    std::atomic<bool> g_stop{ false };
    std::atomic<bool> g_failed{ false };
    std::atomic<bool> g_idlePaused{ false };
    std::atomic<bool> g_decoderDone{ false };
    std::atomic<bool> g_decodeDead{ false };
    std::atomic<bool> g_frameDone{ false };
    std::atomic<bool> g_audioDone{ true };
    std::atomic<bool> g_lowBat{ false };
    std::atomic<bool> g_batHold{ false };
    std::atomic<LONG> g_loopSerial{ 0 };
    std::atomic<LONG> g_seq{ 0 };

    CRITICAL_SECTION g_cs;
    std::vector<uint8_t> g_shared;
    int kW = 960;
    int kH = 540;
    size_t g_frameBytes = 0;
    int g_screenW = 1920;
    int g_screenH = 1080;
    HWND g_wnd = nullptr;
    std::atomic<HHOOK> g_hook{ nullptr };

    std::wstring g_appDataDir;
    std::wstring g_cacheDir;
    std::wstring g_logDir;
    std::wstring g_configPath;

    void InitializePaths() {
        PWSTR programData = nullptr;
        if (SUCCEEDED(SHGetKnownFolderPath(FOLDERID_ProgramData, 0, nullptr, &programData)) && programData) {
            g_appDataDir = std::wstring(programData) + L"\\Sucrose\\LockScreen";
            CoTaskMemFree(programData);
        } else {
            g_appDataDir = L"C:\\ProgramData\\Sucrose\\LockScreen";
        }

        g_cacheDir = g_appDataDir + L"\\Cache";
        g_logDir = g_appDataDir + L"\\Logs";
        g_configPath = g_appDataDir + L"\\config.json";

        CreateDirectoryW(g_appDataDir.c_str(), nullptr);
        CreateDirectoryW(g_cacheDir.c_str(), nullptr);
        CreateDirectoryW(g_logDir.c_str(), nullptr);
    }
}
