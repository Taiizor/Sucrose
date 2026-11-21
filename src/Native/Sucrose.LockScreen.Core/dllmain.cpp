#include "pch.h"
#include "Common.h"
#include "Config.h"
#include "Logger.h"
#include "CacheManager.h"
#include "AudioEngine.h"
#include "VideoDecoder.h"
#include "XamlHook.h"

static HANDLE s_initThread = nullptr;
HMODULE g_hModule = nullptr;

static DWORD WINAPI MainInitThreadProc(LPVOID) {
    Logger::Log(L"=== Sucrose.LockScreen.Core Initializing in LogonUI ===");

    CoreState::InitializePaths();
    ConfigManager::Load(CoreState::g_configPath);

    const auto& cfg = ConfigManager::Current();
    if (!cfg.enabled) {
        Logger::Log(L"Lock screen live wallpaper is disabled in config. Exiting.");
        return 0;
    }

    std::wstring mediaFile = ConfigManager::ResolveMediaFile(cfg.mediaPath, cfg.folderOrder);
    if (mediaFile.empty()) {
        Logger::Log(L"No valid media file resolved from '%s'", cfg.mediaPath.c_str());
        return 0;
    }

    // Canvas size calculation
    int w = cfg.renderWidth;
    if (w < 320) w = 320;
    if (w > 1920) w = 1920;
    CoreState::kW = (w / 2) * 2;

    CoreState::g_screenW = GetSystemMetrics(SM_CXSCREEN);
    CoreState::g_screenH = GetSystemMetrics(SM_CYSCREEN);
    if (CoreState::g_screenW < 320 || CoreState::g_screenH < 200) {
        CoreState::g_screenW = 1920;
        CoreState::g_screenH = 1080;
    }

    CoreState::kH = static_cast<int>(static_cast<long long>(CoreState::kW) * CoreState::g_screenH / CoreState::g_screenW) / 2 * 2;
    if (CoreState::kH < 180) CoreState::kH = 180;
    if (CoreState::kH > 4096) CoreState::kH = 4096;

    CoreState::g_frameBytes = static_cast<size_t>(CoreState::kW) * CoreState::kH * 4;
    CoreState::g_shared.assign(CoreState::g_frameBytes, 0);

    InitializeCriticalSection(&CoreState::g_cs);

    // Try loading first-frame cache
    std::wstring cachePath = CacheManager::GetCacheFilePath(mediaFile, CoreState::kW, CoreState::kH, cfg.fitMode);
    bool cacheHit = CacheManager::LoadFirstFrame(cachePath);
    if (cacheHit) {
        cfg.ApplyColor(reinterpret_cast<uint32_t*>(CoreState::g_shared.data()), static_cast<size_t>(CoreState::kW) * CoreState::kH);
    } else if (cfg.bootStillFrame) {
        CacheManager::LoadLastFrame();
    }

    // Start video decoder
    VideoDecoder::Start(mediaFile);

    // Start audio engine if volume > 0 and not GIF
    size_t dot = mediaFile.find_last_of(L'.');
    std::wstring ext = (dot == std::wstring::npos) ? L"" : mediaFile.substr(dot + 1);
    if (_wcsicmp(ext.c_str(), L"gif") != 0 && cfg.audioVolume > 0 && cfg.speed == 100) {
        AudioEngine::Start(mediaFile, cfg.audioVolume);
    }

    // Start XAML UI hook
    XamlHook::Start();

    return 0;
}

static bool IsTargetProcess() {
    wchar_t exePath[MAX_PATH] = {};
    GetModuleFileNameW(nullptr, exePath, MAX_PATH);
    const wchar_t* base = wcsrchr(exePath, L'\\');
    base = base ? base + 1 : exePath;
    return (_wcsicmp(base, L"LogonUI.exe") == 0);
}

BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call, LPVOID lpReserved) {
    switch (ul_reason_for_call) {
    case DLL_PROCESS_ATTACH:
        g_hModule = hModule;
        DisableThreadLibraryCalls(hModule);
        if (IsTargetProcess()) {
            s_initThread = CreateThread(nullptr, 0, MainInitThreadProc, nullptr, 0, nullptr);
        }
        break;

    case DLL_PROCESS_DETACH:
        if (IsTargetProcess()) {
            CoreState::g_stop = true;
            XamlHook::Stop();
            VideoDecoder::Stop();
            AudioEngine::Stop();

            if (s_initThread) {
                WaitForSingleObject(s_initThread, 1000);
                CloseHandle(s_initThread);
                s_initThread = nullptr;
            }
            DeleteCriticalSection(&CoreState::g_cs);
        }
        break;
    }
    return TRUE;
}
