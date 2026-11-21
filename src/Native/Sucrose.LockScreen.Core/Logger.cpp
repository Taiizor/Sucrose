#include "pch.h"
#include "Logger.h"
#include "Common.h"

namespace Logger {
    static bool s_debugEnabled = true;
    static SRWLOCK s_logLock = SRWLOCK_INIT;

    void SetDebugEnabled(bool enabled) {
        s_debugEnabled = enabled;
    }

    bool IsDebugEnabled() {
        return s_debugEnabled;
    }

    void Log(const wchar_t* fmt, ...) {
        wchar_t msg[1024];
        va_list ap;
        va_start(ap, fmt);
        _vsnwprintf_s(msg, _countof(msg), _TRUNCATE, fmt, ap);
        va_end(ap);

        // Output to DebugView
        OutputDebugStringW(L"[Sucrose.LockScreen.Core] ");
        OutputDebugStringW(msg);
        OutputDebugStringW(L"\n");

        if (!s_debugEnabled || CoreState::g_logDir.empty()) return;

        SYSTEMTIME st;
        GetLocalTime(&st);
        wchar_t line[1200];
        _snwprintf_s(line, _countof(line), _TRUNCATE, L"%02d:%02d:%02d.%03d up=%llus pid=%lu %s\r\n",
                     st.wHour, st.wMinute, st.wSecond, st.wMilliseconds,
                     GetTickCount64() / 1000, GetCurrentProcessId(), msg);

        char u8[3000];
        int n = WideCharToMultiByte(CP_UTF8, 0, line, -1, u8, sizeof(u8), nullptr, nullptr);
        if (n <= 1) return;

        std::wstring logFile = CoreState::g_logDir + L"\\core.log";

        AcquireSRWLockExclusive(&s_logLock);
        WIN32_FILE_ATTRIBUTE_DATA fa;
        if (GetFileAttributesExW(logFile.c_str(), GetFileExInfoStandard, &fa) &&
            (fa.nFileSizeHigh || fa.nFileSizeLow > 2 * 1024 * 1024)) {
            DeleteFileW(logFile.c_str());
        }

        HANDLE f = CreateFileW(logFile.c_str(), FILE_APPEND_DATA, FILE_SHARE_READ | FILE_SHARE_WRITE,
                               nullptr, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
        if (f != INVALID_HANDLE_VALUE) {
            DWORD written = 0;
            WriteFile(f, u8, static_cast<DWORD>(n - 1), &written, nullptr);
            CloseHandle(f);
        }
        ReleaseSRWLockExclusive(&s_logLock);
    }
}
