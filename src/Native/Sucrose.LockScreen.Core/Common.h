#pragma once
#include "pch.h"

// COM interface for direct buffer access to WinRT WriteableBitmap
struct IBufferByteAccess : ::IUnknown {
    virtual HRESULT __stdcall Buffer(uint8_t** value) = 0;
};

static const GUID IID_IBufferByteAccess_ =
    { 0x905a0fef, 0xbc53, 0x11df,
      { 0x8c, 0x49, 0x00, 0x1e, 0x4f, 0xc6, 0x86, 0xda } };

static const GUID kGuidNull = { 0, 0, 0, { 0, 0, 0, 0, 0, 0, 0, 0 } };

// Fit modes
enum FitMode {
    kFitCover = 0,
    kFitFill,
    kFitFit,
    kFitCenter
};

// First-frame cache header structure
struct CacheHdr {
    uint32_t magic; // "LVW4"
    uint32_t w;
    uint32_t h;
    uint32_t pad;
};
static const uint32_t kCacheMagic = 0x4C565734; // "LVW4"
static const uint32_t kOverlayMagic = 0x4C4F5631; // "LOV1"

// Timing helper (QPC)
inline double NowMs() {
    static LARGE_INTEGER f = {};
    if (!f.QuadPart) QueryPerformanceFrequency(&f);
    LARGE_INTEGER c;
    QueryPerformanceCounter(&c);
    return (double)c.QuadPart * 1000.0 / (double)f.QuadPart;
}

// Global state variables
namespace CoreState {
    extern std::atomic<bool> g_stop;
    extern std::atomic<bool> g_failed;
    extern std::atomic<bool> g_idlePaused;
    extern std::atomic<bool> g_decoderDone;
    extern std::atomic<bool> g_decodeDead;
    extern std::atomic<bool> g_frameDone;
    extern std::atomic<bool> g_audioDone;
    extern std::atomic<bool> g_lowBat;
    extern std::atomic<bool> g_batHold;
    extern std::atomic<LONG> g_loopSerial;
    extern std::atomic<LONG> g_seq;

    extern CRITICAL_SECTION g_cs;
    extern std::vector<uint8_t> g_shared;
    extern int kW;
    extern int kH;
    extern size_t g_frameBytes;
    extern int g_screenW;
    extern int g_screenH;
    extern HWND g_wnd;
    extern std::atomic<HHOOK> g_hook;

    extern std::wstring g_appDataDir;
    extern std::wstring g_cacheDir;
    extern std::wstring g_logDir;
    extern std::wstring g_configPath;

    void InitializePaths();
}
