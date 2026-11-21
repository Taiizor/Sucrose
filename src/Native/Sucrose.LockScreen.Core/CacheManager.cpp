#include "pch.h"
#include "CacheManager.h"
#include "Logger.h"

namespace CacheManager {
    static uint64_t Fnv1a(const std::wstring& s) {
        uint64_t h = 1469598103934665603ULL;
        for (wchar_t c : s) {
            h ^= static_cast<uint64_t>(c);
            h *= 1099511628211ULL;
        }
        return h;
    }

    std::wstring GetCacheFilePath(const std::wstring& mediaPath, int width, int height, FitMode fitMode) {
        std::wstring key = mediaPath + L"|" + std::to_wstring(width) + L"|" +
                           std::to_wstring(height) + L"|" + std::to_wstring(static_cast<int>(fitMode)) + L"|raw";
        uint64_t h = Fnv1a(key);
        wchar_t name[64];
        wsprintfW(name, L"frame_%08X%08X.bgra", static_cast<unsigned>(h >> 32), static_cast<unsigned>(h));
        return CoreState::g_cacheDir + L"\\" + name;
    }

    bool LoadFirstFrame(const std::wstring& cachePath) {
        HANDLE f = CreateFileW(cachePath.c_str(), GENERIC_READ, FILE_SHARE_READ,
                               nullptr, OPEN_EXISTING, 0, nullptr);
        if (f == INVALID_HANDLE_VALUE) return false;

        CacheHdr hd = {};
        DWORD got = 0;
        bool ok = ReadFile(f, &hd, sizeof(hd), &got, nullptr) && got == sizeof(hd) &&
                  hd.magic == kCacheMagic &&
                  hd.w == static_cast<uint32_t>(CoreState::kW) &&
                  hd.h == static_cast<uint32_t>(CoreState::kH);

        if (ok) {
            EnterCriticalSection(&CoreState::g_cs);
            ok = ReadFile(f, CoreState::g_shared.data(), static_cast<DWORD>(CoreState::g_frameBytes), &got, nullptr) &&
                 got == CoreState::g_frameBytes;
            LeaveCriticalSection(&CoreState::g_cs);
        }
        CloseHandle(f);

        if (ok) {
            CoreState::g_seq = 1;
            Logger::Log(L"Cache hit: successfully loaded first frame from %s", cachePath.c_str());
        }
        return ok;
    }

    void SaveFirstFrame(const std::wstring& cachePath, const uint8_t* rawPixels) {
        CreateDirectoryW(CoreState::g_cacheDir.c_str(), nullptr);
        HANDLE f = CreateFileW(cachePath.c_str(), GENERIC_WRITE, 0, nullptr,
                               CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
        if (f == INVALID_HANDLE_VALUE) {
            Logger::Log(L"Failed to save cache file (error %lu)", GetLastError());
            return;
        }

        CacheHdr hd = { kCacheMagic, static_cast<uint32_t>(CoreState::kW), static_cast<uint32_t>(CoreState::kH), 0 };
        DWORD written = 0;
        WriteFile(f, &hd, sizeof(hd), &written, nullptr);
        WriteFile(f, rawPixels, static_cast<DWORD>(CoreState::g_frameBytes), &written, nullptr);
        CloseHandle(f);

        Logger::Log(L"First frame saved to cache: %s", cachePath.c_str());
        PruneCache();
    }

    bool LoadLastFrame() {
        std::wstring lastFile = CoreState::g_cacheDir + L"\\last.bgra";
        HANDLE f = CreateFileW(lastFile.c_str(), GENERIC_READ, FILE_SHARE_READ,
                               nullptr, OPEN_EXISTING, 0, nullptr);
        if (f == INVALID_HANDLE_VALUE) return false;

        CacheHdr hd = {};
        DWORD got = 0;
        bool ok = ReadFile(f, &hd, sizeof(hd), &got, nullptr) && got == sizeof(hd) &&
                  hd.magic == kCacheMagic &&
                  hd.w == static_cast<uint32_t>(CoreState::kW) &&
                  hd.h == static_cast<uint32_t>(CoreState::kH);

        if (ok) {
            EnterCriticalSection(&CoreState::g_cs);
            ok = ReadFile(f, CoreState::g_shared.data(), static_cast<DWORD>(CoreState::g_frameBytes), &got, nullptr) &&
                 got == CoreState::g_frameBytes;
            LeaveCriticalSection(&CoreState::g_cs);
        }
        CloseHandle(f);

        if (ok) {
            CoreState::g_seq = 1;
            Logger::Log(L"Loaded boot last frame successfully");
        }
        return ok;
    }

    void SaveLastFrame() {
        std::vector<uint8_t> copy(CoreState::g_frameBytes);
        EnterCriticalSection(&CoreState::g_cs);
        memcpy(copy.data(), CoreState::g_shared.data(), CoreState::g_frameBytes);
        LeaveCriticalSection(&CoreState::g_cs);

        std::wstring lastTmp = CoreState::g_cacheDir + L"\\last.tmp";
        std::wstring lastFile = CoreState::g_cacheDir + L"\\last.bgra";

        HANDLE f = CreateFileW(lastTmp.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS,
                               FILE_ATTRIBUTE_NORMAL, nullptr);
        if (f == INVALID_HANDLE_VALUE) return;

        CacheHdr hd = { kCacheMagic, static_cast<uint32_t>(CoreState::kW), static_cast<uint32_t>(CoreState::kH), 0 };
        DWORD written = 0;
        bool ok = WriteFile(f, &hd, sizeof(hd), &written, nullptr) &&
                  WriteFile(f, copy.data(), static_cast<DWORD>(CoreState::g_frameBytes), &written, nullptr);
        CloseHandle(f);

        if (ok) {
            MoveFileExW(lastTmp.c_str(), lastFile.c_str(), MOVEFILE_REPLACE_EXISTING);
        }
    }

    void PruneCache() {
        std::wstring pat = CoreState::g_cacheDir + L"\\frame_*.bgra";
        WIN32_FIND_DATAW fd;
        HANDLE h = FindFirstFileW(pat.c_str(), &fd);
        if (h == INVALID_HANDLE_VALUE) return;

        struct Entry { std::wstring path; ULONGLONG t; };
        std::vector<Entry> entries;
        FILETIME nowFt;
        GetSystemTimeAsFileTime(&nowFt);
        ULONGLONG now = (static_cast<ULONGLONG>(nowFt.dwHighDateTime) << 32) | nowFt.dwLowDateTime;
        const ULONGLONG kMaxAge = 30ULL * 24 * 3600 * 10000000ULL; // 30 days in 100-ns units

        do {
            std::wstring p = CoreState::g_cacheDir + L"\\" + fd.cFileName;
            ULONGLONG t = (static_cast<ULONGLONG>(fd.ftLastWriteTime.dwHighDateTime) << 32) |
                          fd.ftLastWriteTime.dwLowDateTime;
            if (now > t && now - t > kMaxAge) {
                DeleteFileW(p.c_str());
            } else {
                entries.push_back({ p, t });
            }
        } while (FindNextFileW(h, &fd));
        FindClose(h);

        if (entries.size() <= 5) return;
        std::sort(entries.begin(), entries.end(), [](const Entry& a, const Entry& b) {
            return a.t > b.t;
        });

        // Keep 5 newest, delete older ones
        for (size_t i = 5; i < entries.size(); i++) {
            DeleteFileW(entries[i].path.c_str());
        }
    }
}
