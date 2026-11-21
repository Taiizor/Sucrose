#include "pch.h"
#include "XamlHook.h"
#include "Common.h"
#include "Config.h"
#include "Logger.h"

using namespace winrt::Windows::UI::Xaml;

namespace XamlHook {
    static HANDLE s_workerThread = nullptr;
    static LONG s_lastSeq = 0;
    static uint8_t* s_pixels = nullptr;
    static std::atomic<bool> s_teardownRequested{ false };
    static std::atomic<bool> s_tornDown{ false };

    static Media::Imaging::WriteableBitmap s_wb{ nullptr };
    static Controls::Image s_imgA{ nullptr };
    static Controls::Image s_imgB{ nullptr };
    static Media::ImageSource s_origSrcA{ nullptr };
    static Media::ImageSource s_origSrcB{ nullptr };
    static Media::Stretch s_origStrA = Media::Stretch::Uniform;
    static Media::Stretch s_origStrB = Media::Stretch::Uniform;
    static bool s_savedA = false, s_savedB = false;

    static DWORD s_initTick = 0;
    static DWORD s_lastFind = 0;
    static DWORD s_lastInput = 0;
    static DWORD s_missSince = 0;
    static int s_missTries = 0;
    static bool s_everFound = false;
    static bool s_pwScene = false;
    static DWORD s_pwShownTick = 0;
    static DWORD s_pwGoneTick = 0;

    static int s_errCount = 0;
    static DWORD s_errWindowStart = 0;
    static DWORD s_lastErrTick = 0;
    static const int kMaxErrors = 3;
    static const DWORD kErrWindowMs = 30000;
    static const DWORD kErrThrottleMs = 500;
    static const DWORD kFailSafeMs = 30000;
    static const int kMinMissTries = 20;

    static void TeardownInternal() {
        Logger::Log(L"XamlHook: Restoring original wallpaper and releasing XAML resources...");
        try {
            if (s_imgA && s_savedA && s_imgA.Source() == s_wb) {
                s_imgA.Source(s_origSrcA);
                s_imgA.Stretch(s_origStrA);
            }
            if (s_imgB && s_savedB && s_imgB.Source() == s_wb) {
                s_imgB.Source(s_origSrcB);
                s_imgB.Stretch(s_origStrB);
            }
        } catch (...) {}

        s_imgA = nullptr;
        s_imgB = nullptr;
        s_origSrcA = nullptr;
        s_origSrcB = nullptr;
        s_wb = nullptr;
        s_pixels = nullptr;
        s_tornDown = true;
    }

    void TeardownOnUIThread() {
        if (!s_tornDown) {
            TeardownInternal();
        }
    }

    void TripFailSafe(const wchar_t* reason) {
        if (CoreState::g_failed.exchange(true)) return;
        Logger::Log(L"FAIL-SAFE TRIPPED: %s -> Restoring original wallpaper and stopping core", reason);
        TeardownOnUIThread();
        CoreState::g_stop = true;
    }

    static void OnStepError(const wchar_t* what) {
        DWORD now = GetTickCount();
        if (s_pwGoneTick && now - s_pwGoneTick < 10000) return;
        if (s_lastErrTick && now - s_lastErrTick < kErrThrottleMs) return;
        s_lastErrTick = now;

        if (s_errCount == 0 || now - s_errWindowStart > kErrWindowMs) {
            s_errWindowStart = now;
            s_errCount = 0;
        }
        s_errCount++;
        Logger::Log(L"ERROR #%d: %s", s_errCount, what);

        if (s_errCount >= kMaxErrors) {
            TripFailSafe(L"Too many exceptions while manipulating XAML UI");
        }
    }

    static void EnsureBitmap() {
        if (s_wb) return;
        int kW = CoreState::kW;
        int kH = CoreState::kH;

        Media::Imaging::WriteableBitmap wb(kW, kH);
        winrt::com_ptr<IBufferByteAccess> bba;
        winrt::check_hresult(winrt::get_unknown(wb.PixelBuffer())->QueryInterface(
            IID_IBufferByteAccess_, bba.put_void()));

        uint8_t* p = nullptr;
        winrt::check_hresult(bba->Buffer(&p));

        uint32_t* px = reinterpret_cast<uint32_t*>(p);
        for (int i = 0; i < kW * kH; i++) {
            px[i] = 0xFF000000u;
        }

        s_pixels = p;
        s_wb = wb;
        s_wb.Invalidate();
        Logger::Log(L"WriteableBitmap %dx%d created successfully", kW, kH);
    }

    struct SceneNodes {
        DependencyObject msg{ nullptr };
        DependencyObject lock{ nullptr };
        DependencyObject cred{ nullptr };
        DependencyObject imgA{ nullptr };
        DependencyObject imgB{ nullptr };
    };

    static void CollectNodes(DependencyObject const& o, int depth, SceneNodes& s) {
        if (depth > 60) return;
        if (auto fe = o.try_as<FrameworkElement>()) {
            auto n = fe.Name();
            if (!n.empty()) {
                if (!s.msg && n == L"DisplayMessageUserControl") s.msg = o;
                else if (!s.lock && n == L"LockScreenUserControl") s.lock = o;
                else if (!s.cred && n == L"RequestCredentialUserControl") s.cred = o;
                else if (!s.imgA && n == L"LockLogonImage") s.imgA = o;
                else if (!s.imgB && n == L"SwapLockLogonImage") s.imgB = o;
            }
        }
        int cnt = Media::VisualTreeHelper::GetChildrenCount(o);
        for (int i = 0; i < cnt; i++) {
            CollectNodes(Media::VisualTreeHelper::GetChild(o, i), depth + 1, s);
        }
    }

    static bool Attach(Controls::Image const& im, bool isA) {
        auto cur = im.Source();
        if (cur && cur == s_wb) return true;

        if (isA) {
            s_origSrcA = cur;
            s_origStrA = im.Stretch();
            s_savedA = true;
        } else {
            s_origSrcB = cur;
            s_origStrB = im.Stretch();
            s_savedB = true;
        }

        im.Source(s_wb);
        im.Stretch(Media::Stretch::UniformToFill);
        auto after = im.Source();
        if (!(after && after == s_wb)) return false;

        Logger::Log(L"Attached to %s", isA ? L"LockLogonImage" : L"SwapLockLogonImage");
        return true;
    }

    static void Step() {
        if (CoreState::g_failed) return;

        try {
            DWORD now = GetTickCount();
            EnsureBitmap();

            if (CoreState::g_seq == 0) {
                if (CoreState::g_decodeDead || (now - s_initTick >= kFailSafeMs)) {
                    TripFailSafe(CoreState::g_decodeDead ? L"Decoder failed, no frame produced"
                                                         : L"Timeout before first frame produced");
                }
                return;
            }

            if (CoreState::g_seq != s_lastSeq && TryEnterCriticalSection(&CoreState::g_cs)) {
                memcpy(s_pixels, CoreState::g_shared.data(), CoreState::g_frameBytes);
                s_lastSeq = CoreState::g_seq;
                LeaveCriticalSection(&CoreState::g_cs);
                s_wb.Invalidate();
            }

            bool both = (s_imgA && s_imgB);
            DWORD interval = both ? 1000u : (now - s_initTick < 5000 ? 30u : 100u);

            if (now - s_lastFind >= interval) {
                s_lastFind = now;
                auto window = Window::Current();
                auto content = window ? window.Content() : nullptr;
                if (content) {
                    SceneNodes sn;
                    CollectNodes(content, 0, sn);

                    auto a = sn.imgA;
                    auto b = sn.imgB;
                    s_imgA = a ? a.try_as<Controls::Image>() : nullptr;
                    s_imgB = b ? b.try_as<Controls::Image>() : nullptr;

                    // Password screen detection
                    auto credFe = sn.cred ? sn.cred.try_as<FrameworkElement>() : nullptr;
                    auto lockFe = sn.lock ? sn.lock.try_as<FrameworkElement>() : nullptr;
                    bool credVis = credFe && credFe.Visibility() == Visibility::Visible && credFe.ActualWidth() > 0;
                    bool pw = credVis && lockFe && lockFe.Visibility() != Visibility::Visible;

                    if (pw != s_pwScene) {
                        if (!s_pwScene && pw) s_pwShownTick = now;
                        if (s_pwScene && !pw) s_pwGoneTick = now;
                        s_pwScene = pw;
                    }
                } else {
                    s_imgA = nullptr;
                    s_imgB = nullptr;
                }

                if (s_imgA || s_imgB) {
                    s_everFound = true;
                    s_missSince = 0;
                    s_missTries = 0;
                } else {
                    if (!s_missSince) s_missSince = now ? now : 1;
                    s_missTries++;
                    if (!s_everFound && s_missTries >= kMinMissTries && (now - s_missSince >= kFailSafeMs)) {
                        TripFailSafe(L"LockLogonImage elements not found (UI tree changed)");
                        return;
                    }
                }
            }

            // Playback hold logic (idle / password screen / battery)
            const auto& cfg = ConfigManager::Current();
            bool idle = false;
            if (cfg.idlePauseSeconds > 0) {
                if (!s_lastInput) s_lastInput = now;
                idle = (now - s_lastInput >= cfg.idlePauseSeconds * 1000u);
            }

            bool hold = idle || (!cfg.playOnPasswordScreen && s_pwScene) || CoreState::g_batHold.load();
            if (hold != CoreState::g_idlePaused.load()) {
                CoreState::g_idlePaused = hold;
                Logger::Log(L"Playback hold: %s (idle=%d, pwScene=%d)", hold ? L"ON" : L"OFF", idle ? 1 : 0, s_pwScene ? 1 : 0);
            }

            bool ok = true;
            if (s_imgA) ok = Attach(s_imgA, true) && ok;
            if (s_imgB) ok = Attach(s_imgB, false) && ok;
            if (!ok) OnStepError(L"Attach failed: Image Source could not be updated");

        } catch (winrt::hresult_error const& e) {
            s_imgA = nullptr;
            s_imgB = nullptr;
            OnStepError(e.message().c_str());
        } catch (...) {
            s_imgA = nullptr;
            s_imgB = nullptr;
            OnStepError(L"Non-WinRT exception in Step()");
        }
    }

    static LRESULT CALLBACK GetMsgProc(int code, WPARAM w, LPARAM l) {
        if (code >= 0) {
            if (s_teardownRequested) {
                if (!s_tornDown) TeardownInternal();
            } else if (!CoreState::g_stop && !CoreState::g_failed) {
                MSG* m = reinterpret_cast<MSG*>(l);
                if (m && (m->message == WM_KEYDOWN || m->message == WM_MOUSEMOVE ||
                          m->message == WM_LBUTTONDOWN || m->message == 0x0246 /* WM_POINTERDOWN */)) {
                    s_lastInput = GetTickCount();
                }
                Step();
            }
        }
        return CallNextHookEx(CoreState::g_hook.load(), code, w, l);
    }

    static BOOL CALLBACK EnumLogonUIWindows(HWND hwnd, LPARAM) {
        DWORD pid = 0;
        GetWindowThreadProcessId(hwnd, &pid);
        if (pid != GetCurrentProcessId()) return TRUE;

        wchar_t cls[128] = {};
        GetClassNameW(hwnd, cls, 128);
        if (wcscmp(cls, L"LogonUI Logon Window") == 0) {
            CoreState::g_wnd = hwnd;
            return FALSE;
        }
        return TRUE;
    }

    static DWORD WINAPI WorkerThreadProc(LPVOID) {
        s_initTick = GetTickCount();
        Logger::Log(L"Worker thread started. Searching for 'LogonUI Logon Window'...");

        for (int i = 0; i < 3000 && !CoreState::g_stop; i++) {
            CoreState::g_wnd = nullptr;
            EnumWindows(EnumLogonUIWindows, 0);
            if (CoreState::g_wnd) break;
            Sleep(10);
        }

        if (!CoreState::g_wnd) {
            Logger::Log(L"LogonUI Logon Window not found after timeout");
            return 0;
        }

        DWORD tid = GetWindowThreadProcessId(CoreState::g_wnd, nullptr);
        HHOOK hk = SetWindowsHookExW(WH_GETMESSAGE, GetMsgProc, nullptr, tid);
        CoreState::g_hook = hk;

        if (!hk) {
            Logger::Log(L"SetWindowsHookExW failed (error %lu)", GetLastError());
            CoreState::g_stop = true;
            return 0;
        }

        Logger::Log(L"Hook successfully installed on LogonUI UI Thread: %lu (HWND: %p)", tid, CoreState::g_wnd);

        while (!CoreState::g_stop && IsWindow(CoreState::g_wnd)) {
            PostMessageW(CoreState::g_wnd, WM_NULL, 0, 0);
            Sleep((CoreState::g_idlePaused.load() || CoreState::g_decoderDone.load()) ? 1000 : 20);
        }

        HHOOK h = CoreState::g_hook.exchange(nullptr);
        if (h) UnhookWindowsHookEx(h);
        CoreState::g_wnd = nullptr;
        Logger::Log(L"Worker thread exiting.");
        return 0;
    }

    bool Start() {
        s_workerThread = CreateThread(nullptr, 0, WorkerThreadProc, nullptr, 0, nullptr);
        return s_workerThread != nullptr;
    }

    void Stop() {
        s_teardownRequested = true;
        HWND w = CoreState::g_wnd;
        if (w && IsWindow(w)) {
            for (int i = 0; i < 60 && !s_tornDown; i++) {
                PostMessageW(w, WM_NULL, 0, 0);
                Sleep(50);
            }
        }

        CoreState::g_stop = true;
        if (s_workerThread) {
            WaitForSingleObject(s_workerThread, 3000);
            CloseHandle(s_workerThread);
            s_workerThread = nullptr;
        }

        HHOOK h = CoreState::g_hook.exchange(nullptr);
        if (h) UnhookWindowsHookEx(h);
    }
}
