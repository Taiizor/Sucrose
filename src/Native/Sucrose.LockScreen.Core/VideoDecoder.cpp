#include "pch.h"
#include "VideoDecoder.h"
#include "Common.h"
#include "Config.h"
#include "CacheManager.h"
#include "Logger.h"

template <class T> static void SafeRelease(T*& p) {
    if (p) { p->Release(); p = nullptr; }
}

static const GUID kGuidWicPBGRA =
    { 0x6fddc324, 0x4e03, 0x4bfe, { 0xb1, 0x85, 0x3d, 0x77, 0x76, 0x8d, 0xc9, 0x10 } };
static const CLSID kClsidWicFactory =
    { 0xcacaf262, 0x9370, 0x4615, { 0xa1, 0x3b, 0x9f, 0x55, 0x39, 0xda, 0x4c, 0x0a } };

struct GpuState {
    ID3D11Device* dev = nullptr;
    ID3D11DeviceContext* ctx = nullptr;
    IMFDXGIDeviceManager* mgr = nullptr;
    ID3D11Texture2D* staging = nullptr;
    UINT stW = 0, stH = 0;
    DXGI_FORMAT stFmt = DXGI_FORMAT_UNKNOWN;

    bool Init() {
        D3D_FEATURE_LEVEL fl;
        UINT flags = D3D11_CREATE_DEVICE_VIDEO_SUPPORT | D3D11_CREATE_DEVICE_BGRA_SUPPORT;
        HRESULT hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, flags,
                                       nullptr, 0, D3D11_SDK_VERSION, &dev, &fl, &ctx);
        if (FAILED(hr) || !dev || !ctx) {
            Logger::Log(L"GPU: D3D11CreateDevice failed (hr=0x%08X)", static_cast<unsigned>(hr));
            return false;
        }

        ID3D11Multithread* mtp = nullptr;
        if (SUCCEEDED(dev->QueryInterface(__uuidof(ID3D11Multithread), reinterpret_cast<void**>(&mtp))) && mtp) {
            mtp->SetMultithreadProtected(TRUE);
            mtp->Release();
        }

        UINT token = 0;
        hr = MFCreateDXGIDeviceManager(&token, &mgr);
        if (FAILED(hr) || !mgr) {
            Logger::Log(L"GPU: MFCreateDXGIDeviceManager failed (hr=0x%08X)", static_cast<unsigned>(hr));
            return false;
        }

        hr = mgr->ResetDevice(dev, token);
        if (FAILED(hr)) {
            Logger::Log(L"GPU: ResetDevice failed (hr=0x%08X)", static_cast<unsigned>(hr));
            return false;
        }
        return true;
    }

    bool Map(IMFSample* s, D3D11_MAPPED_SUBRESOURCE& m, UINT& w, UINT& h) {
        IMFMediaBuffer* b = nullptr;
        if (FAILED(s->GetBufferByIndex(0, &b)) || !b) return false;
        bool ok = false;
        IMFDXGIBuffer* db = nullptr;
        if (SUCCEEDED(b->QueryInterface(__uuidof(IMFDXGIBuffer), reinterpret_cast<void**>(&db))) && db) {
            ID3D11Texture2D* tex = nullptr;
            if (SUCCEEDED(db->GetResource(__uuidof(ID3D11Texture2D), reinterpret_cast<void**>(&tex))) && tex) {
                UINT sub = 0;
                db->GetSubresourceIndex(&sub);
                D3D11_TEXTURE2D_DESC d;
                tex->GetDesc(&d);
                if (!staging || stW != d.Width || stH != d.Height || stFmt != d.Format) {
                    SafeRelease(staging);
                    D3D11_TEXTURE2D_DESC sd = d;
                    sd.Usage = D3D11_USAGE_STAGING;
                    sd.BindFlags = 0;
                    sd.MiscFlags = 0;
                    sd.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
                    sd.ArraySize = 1;
                    sd.MipLevels = 1;
                    if (SUCCEEDED(dev->CreateTexture2D(&sd, nullptr, &staging))) {
                        stW = d.Width; stH = d.Height; stFmt = d.Format;
                    }
                }
                if (staging) {
                    ctx->CopySubresourceRegion(staging, 0, 0, 0, 0, tex, sub, nullptr);
                    if (SUCCEEDED(ctx->Map(staging, 0, D3D11_MAP_READ, 0, &m))) {
                        w = d.Width;
                        h = d.Height;
                        ok = true;
                    }
                }
                tex->Release();
            }
            db->Release();
        }
        b->Release();
        return ok;
    }

    void Unmap() {
        if (ctx && staging) ctx->Unmap(staging, 0);
    }

    ~GpuState() {
        SafeRelease(staging);
        SafeRelease(mgr);
        SafeRelease(ctx);
        SafeRelease(dev);
    }
};

static bool RenderFrame(const BYTE* data, size_t stride, bool flip, int sw, int sh,
                        uint8_t* out, std::vector<int>& xmap, FitMode fitMode) {
    int kW = CoreState::kW;
    int kH = CoreState::kH;
    if (sw <= 0 || sh <= 0 || stride < static_cast<size_t>(sw * 4)) return false;

    float scX = 1.0f, scY = 1.0f;
    switch (fitMode) {
    case kFitFill:
        scX = static_cast<float>(kW) / sw;
        scY = static_cast<float>(kH) / sh;
        break;
    case kFitFit:
        scX = scY = std::min(static_cast<float>(kW) / sw, static_cast<float>(kH) / sh);
        break;
    case kFitCenter:
        scX = scY = static_cast<float>(kW) / static_cast<float>(CoreState::g_screenW);
        break;
    default: // cover
        scX = scY = std::max(static_cast<float>(kW) / sw, static_cast<float>(kH) / sh);
        break;
    }

    const uint32_t kBlack = 0xFF000000u;
    for (int x = 0; x < kW; x++) {
        int sx = static_cast<int>(std::floor(((static_cast<float>(x) + 0.5f - kW * 0.5f) / scX) + sw * 0.5f));
        xmap[x] = (sx >= 0 && sx < sw) ? sx : -1;
    }

    for (int y = 0; y < kH; y++) {
        uint32_t* dst = reinterpret_cast<uint32_t*>(out + static_cast<size_t>(y) * kW * 4);
        int sy = static_cast<int>(std::floor(((static_cast<float>(y) + 0.5f - kH * 0.5f) / scY) + sh * 0.5f));
        if (sy < 0 || sy >= sh) {
            for (int x = 0; x < kW; x++) dst[x] = kBlack;
            continue;
        }
        int row = flip ? (sh - 1 - sy) : sy;
        const uint32_t* src = reinterpret_cast<const uint32_t*>(data + static_cast<size_t>(row) * stride);
        for (int x = 0; x < kW; x++) {
            dst[x] = xmap[x] < 0 ? kBlack : (src[xmap[x]] | kBlack);
        }
    }
    return true;
}

namespace VideoDecoder {
    static HANDLE s_decThread = nullptr;
    static std::wstring s_mediaPath;
    static bool s_isGif = false;
    static std::atomic<bool> s_decPublished{ false };

    static DWORD DecodeRun(bool gpu) {
        struct ExitGuard {
            bool& gpuFlag;
            ExitGuard(bool& g) : gpuFlag(g) {}
            ~ExitGuard() {
                if (gpuFlag && !s_decPublished && !CoreState::g_stop) return;
                if (CoreState::g_seq == 0) CoreState::g_decodeDead = true;
                CoreState::g_decoderDone = true;
            }
        } exitGuard(gpu);

        HRESULT hrCo = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
        struct CoGuard {
            bool on;
            ~CoGuard() { if (on) CoUninitialize(); }
        } coGuard{ SUCCEEDED(hrCo) };

        timeBeginPeriod(1);
        struct TimePeriodGuard {
            ~TimePeriodGuard() { timeEndPeriod(1); }
        } timeGuard;

        if (FAILED(MFStartup(MF_VERSION))) {
            Logger::Log(L"MFStartup failed");
            return 1;
        }

        IMFAttributes* at = nullptr;
        MFCreateAttributes(&at, 1);
        if (at) at->SetUINT32(MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, TRUE);

        GpuState gs;
        bool gpuActive = false;
        if (gpu) {
            if (at && gs.Init()) {
                at->SetUnknown(MF_SOURCE_READER_D3D_MANAGER, gs.mgr);
                at->SetUINT32(MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, TRUE);
                gpuActive = true;
            } else {
                Logger::Log(L"Hardware decoding unavailable, fallback to software");
                gpu = false;
            }
        }

        IMFSourceReader* rd = nullptr;
        HRESULT hr = MFCreateSourceReaderFromURL(s_mediaPath.c_str(), at, &rd);
        if (at) at->Release();

        if (FAILED(hr) || !rd) {
            Logger::Log(L"Failed to open video source reader: %s (hr=0x%08X)", s_mediaPath.c_str(), static_cast<unsigned>(hr));
            MFShutdown();
            return 1;
        }

        const DWORD VS = static_cast<DWORD>(MF_SOURCE_READER_FIRST_VIDEO_STREAM);
        rd->SetStreamSelection(static_cast<DWORD>(MF_SOURCE_READER_ALL_STREAMS), FALSE);
        rd->SetStreamSelection(VS, TRUE);

        IMFMediaType* mt = nullptr;
        if (FAILED(MFCreateMediaType(&mt)) || !mt) {
            rd->Release();
            MFShutdown();
            return 1;
        }
        mt->SetGUID(MF_MT_MAJOR_TYPE, MFMediaType_Video);
        mt->SetGUID(MF_MT_SUBTYPE, gpu ? MFVideoFormat_ARGB32 : MFVideoFormat_RGB32);

        // Pre-downscale in Media Foundation for large sources
        {
            IMFMediaType* nat = nullptr;
            UINT32 nw = 0, nh = 0;
            if (SUCCEEDED(rd->GetNativeMediaType(VS, 0, &nat)) && nat) {
                MFGetAttributeSize(nat, MF_MT_FRAME_SIZE, &nw, &nh);
                nat->Release();
            }
            if (nw > 0 && nh > 0) {
                const auto& cfg = ConfigManager::Current();
                float sc = (cfg.fitMode == kFitFit)
                    ? std::min(static_cast<float>(CoreState::kW) / nw, static_cast<float>(CoreState::kH) / nh)
                    : std::max(static_cast<float>(CoreState::kW) / nw, static_cast<float>(CoreState::kH) / nh);

                if (sc < 0.98f) {
                    UINT32 tw = (static_cast<UINT32>(nw * sc + 0.5f) + 1) & ~1u;
                    UINT32 th = (static_cast<UINT32>(nh * sc + 0.5f) + 1) & ~1u;
                    if (tw >= 2 && th >= 2) {
                        MFSetAttributeSize(mt, MF_MT_FRAME_SIZE, tw, th);
                    }
                }
            }
        }

        hr = rd->SetCurrentMediaType(VS, nullptr, mt);
        if (FAILED(hr)) {
            mt->DeleteItem(MF_MT_FRAME_SIZE);
            hr = rd->SetCurrentMediaType(VS, nullptr, mt);
        }
        mt->Release();
        if (FAILED(hr)) {
            Logger::Log(L"Unsupported video format (hr=0x%08X)", static_cast<unsigned>(hr));
            rd->Release();
            MFShutdown();
            return 1;
        }

        UINT32 sw = 0, sh = 0, visH = 0;
        LONG stride = 0;
        auto readType = [&]() {
            IMFMediaType* cur = nullptr;
            if (SUCCEEDED(rd->GetCurrentMediaType(VS, &cur)) && cur) {
                MFGetAttributeSize(cur, MF_MT_FRAME_SIZE, &sw, &sh);
                stride = static_cast<LONG>(MFGetAttributeUINT32(cur, MF_MT_DEFAULT_STRIDE, sw * 4));
                visH = sh;
                MFVideoArea ap = {};
                if (SUCCEEDED(cur->GetBlob(MF_MT_MINIMUM_DISPLAY_APERTURE, reinterpret_cast<UINT8*>(&ap), sizeof(ap), nullptr)) &&
                    ap.Area.cy > 0 && static_cast<UINT32>(ap.Area.cy) <= sh && ap.OffsetY.value == 0) {
                    visH = static_cast<UINT32>(ap.Area.cy);
                }
                cur->Release();
            }
        };
        readType();
        if (!sw || !sh) { rd->Release(); MFShutdown(); return 1; }

        std::vector<uint8_t> tmp(CoreState::g_frameBytes);
        std::vector<int> xmap(CoreState::kW);
        std::vector<uint8_t> rawFirst;

        ULONGLONG t0 = GetTickCount64();
        bool firstPublished = false;
        int gpuMiss = 0;
        const auto& cfg = ConfigManager::Current();

        while (!CoreState::g_stop) {
            if (CoreState::g_idlePaused.load() && firstPublished) {
                Sleep(100);
                continue;
            }

            DWORD actual = 0, flags = 0;
            LONGLONG ts = 0;
            IMFSample* sample = nullptr;
            hr = rd->ReadSample(VS, 0, &actual, &flags, &ts, &sample);
            if (FAILED(hr) || (flags & MF_SOURCE_READERF_ERROR)) {
                if (sample) sample->Release();
                break;
            }
            if (flags & MF_SOURCE_READERF_ENDOFSTREAM) {
                PROPVARIANT pv;
                PropVariantInit(&pv);
                pv.vt = VT_I8;
                pv.hVal.QuadPart = 0;
                rd->SetCurrentPosition(kGuidNull, pv);
                t0 = GetTickCount64();
                CoreState::g_loopSerial++;
                if (sample) sample->Release();
                continue;
            }
            if (flags & MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED) readType();
            if (!sample) continue;

            bool have = false;
            if (gpu) {
                D3D11_MAPPED_SUBRESOURCE mp = {};
                UINT tw = 0, th = 0;
                if (gs.Map(sample, mp, tw, th)) {
                    if (mp.pData && mp.RowPitch >= sw * 4 && th > 0) {
                        have = RenderFrame(reinterpret_cast<const BYTE*>(mp.pData), static_cast<size_t>(mp.RowPitch), false,
                                           static_cast<int>(sw), static_cast<int>(visH ? visH : sh),
                                           tmp.data(), xmap, cfg.fitMode);
                    }
                    gs.Unmap();
                }
            }

            if (!gpu) {
                IMFMediaBuffer* buf = nullptr;
                if (SUCCEEDED(sample->ConvertToContiguousBuffer(&buf)) && buf) {
                    BYTE* data = nullptr;
                    DWORD maxLen = 0, curLen = 0;
                    if (SUCCEEDED(buf->Lock(&data, &maxLen, &curLen)) && data) {
                        bool flip = stride < 0;
                        size_t absStride = stride < 0 ? static_cast<size_t>(-stride) : static_cast<size_t>(stride);
                        if (absStride >= static_cast<size_t>(sw * 4) && absStride * sh <= curLen) {
                            have = RenderFrame(data, absStride, flip, static_cast<int>(sw), static_cast<int>(visH ? visH : sh),
                                               tmp.data(), xmap, cfg.fitMode);
                        }
                        buf->Unlock();
                    }
                    buf->Release();
                }
            }
            sample->Release();

            if (!have) {
                if (gpu && !firstPublished && ++gpuMiss >= 10) {
                    Logger::Log(L"GPU decode produced no frames, falling back to software");
                    break;
                }
                continue;
            }

            if (rawFirst.empty()) rawFirst = tmp;
            cfg.ApplyColor(reinterpret_cast<uint32_t*>(tmp.data()), static_cast<size_t>(CoreState::kW) * CoreState::kH);

            // Frame presentation timing
            ULONGLONG mediaMs = static_cast<ULONGLONG>(ts / 10000);
            ULONGLONG due = t0 + mediaMs * 100ULL / static_cast<ULONGLONG>(cfg.speed > 0 ? cfg.speed : 100);
            for (;;) {
                ULONGLONG now = GetTickCount64();
                if (CoreState::g_stop || now >= due) {
                    if (now > due + 250) t0 += (now - due);
                    break;
                }
                Sleep(static_cast<DWORD>(std::min<ULONGLONG>(due - now, 4)));
            }
            if (CoreState::g_stop) break;

            EnterCriticalSection(&CoreState::g_cs);
            memcpy(CoreState::g_shared.data(), tmp.data(), CoreState::g_frameBytes);
            CoreState::g_seq++;
            LeaveCriticalSection(&CoreState::g_cs);

            if (CoreState::g_wnd) {
                PostMessageW(CoreState::g_wnd, WM_NULL, 0, 0);
            }

            if (!firstPublished) {
                firstPublished = true;
                s_decPublished = true;
                CoreState::g_frameDone = true;
                std::wstring cachePath = CacheManager::GetCacheFilePath(s_mediaPath, CoreState::kW, CoreState::kH, cfg.fitMode);
                CacheManager::SaveFirstFrame(cachePath, rawFirst.data());
                Logger::Log(gpu ? L"Decoder: Hardware (D3D11) active" : L"Decoder: Software active");
            }
        }

        rd->Release();
        MFShutdown();
        return 0;
    }

    static UINT WicU16(IWICMetadataQueryReader* q, const wchar_t* name, UINT def) {
        UINT v = def;
        if (!q) return v;
        PROPVARIANT pv;
        PropVariantInit(&pv);
        if (SUCCEEDED(q->GetMetadataByName(name, &pv))) {
            if (pv.vt == VT_UI2) v = pv.uiVal;
            else if (pv.vt == VT_UI1) v = pv.bVal;
        }
        PropVariantClear(&pv);
        return v;
    }

    static DWORD GifRun() {
        HRESULT hrCo = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
        struct CoGuard {
            bool on;
            ~CoGuard() { if (on) CoUninitialize(); }
        } coGuard{ SUCCEEDED(hrCo) };

        IWICImagingFactory* fac = nullptr;
        IWICBitmapDecoder* dec = nullptr;
        HRESULT hr = CoCreateInstance(kClsidWicFactory, nullptr, CLSCTX_INPROC_SERVER,
                                      __uuidof(IWICImagingFactory), reinterpret_cast<void**>(&fac));
        if (SUCCEEDED(hr) && fac) {
            hr = fac->CreateDecoderFromFilename(s_mediaPath.c_str(), nullptr, GENERIC_READ,
                                                WICDecodeMetadataCacheOnDemand, &dec);
        }

        UINT count = 0;
        if (SUCCEEDED(hr) && dec) dec->GetFrameCount(&count);
        if (FAILED(hr) || !dec || count == 0) {
            Logger::Log(L"GIF: cannot open or no frames: %s", s_mediaPath.c_str());
            SafeRelease(dec);
            SafeRelease(fac);
            CoreState::g_decodeDead = true;
            CoreState::g_decoderDone = true;
            return 1;
        }

        UINT lw = 0, lh = 0;
        IWICBitmapFrameDecode* f0 = nullptr;
        if (SUCCEEDED(dec->GetFrame(0, &f0)) && f0) {
            f0->GetSize(&lw, &lh);
            f0->Release();
        }
        if (!lw || !lh) { SafeRelease(dec); SafeRelease(fac); return 1; }

        std::vector<uint32_t> canvas(static_cast<size_t>(lw) * lh, 0), prev, frame;
        std::vector<uint8_t> tmp(CoreState::g_frameBytes), rawFirst;
        std::vector<int> xmap(CoreState::kW);
        ULONGLONG nextDue = GetTickCount64();
        bool firstPublished = false;
        UINT idx = 0;
        const auto& cfg = ConfigManager::Current();

        while (!CoreState::g_stop) {
            if (CoreState::g_idlePaused.load() && firstPublished) {
                Sleep(100);
                nextDue = GetTickCount64();
                continue;
            }

            IWICBitmapFrameDecode* fr = nullptr;
            if (FAILED(dec->GetFrame(idx, &fr)) || !fr) break;
            IWICMetadataQueryReader* q = nullptr;
            fr->GetMetadataQueryReader(&q);
            UINT left = WicU16(q, L"/imgdesc/Left", 0);
            UINT top = WicU16(q, L"/imgdesc/Top", 0);
            UINT delay = WicU16(q, L"/grctlext/Delay", 10);
            UINT disp = WicU16(q, L"/grctlext/Disposal", 0);
            SafeRelease(q);

            UINT fw = 0, fh = 0;
            bool ok = false;
            IWICFormatConverter* cv = nullptr;
            if (SUCCEEDED(fac->CreateFormatConverter(&cv)) && cv &&
                SUCCEEDED(cv->Initialize(fr, kGuidWicPBGRA, WICBitmapDitherTypeNone, nullptr, 0.0, WICBitmapPaletteTypeCustom)) &&
                SUCCEEDED(cv->GetSize(&fw, &fh)) && fw && fh) {
                frame.resize(static_cast<size_t>(fw) * fh);
                ok = SUCCEEDED(cv->CopyPixels(nullptr, fw * 4, static_cast<UINT>(frame.size() * 4), reinterpret_cast<BYTE*>(frame.data())));
            }
            SafeRelease(cv);
            SafeRelease(fr);
            if (!ok) break;

            if (disp == 3) prev = canvas;
            for (UINT y = 0; y < fh && top + y < lh; y++) {
                for (UINT x = 0; x < fw && left + x < lw; x++) {
                    uint32_t p = frame[static_cast<size_t>(y) * fw + x];
                    if (p >> 24) canvas[static_cast<size_t>(top + y) * lw + (left + x)] = p;
                }
            }

            if (!RenderFrame(reinterpret_cast<const BYTE*>(canvas.data()), static_cast<size_t>(lw) * 4, false,
                            static_cast<int>(lw), static_cast<int>(lh), tmp.data(), xmap, cfg.fitMode))
                break;

            if (rawFirst.empty()) rawFirst = tmp;
            cfg.ApplyColor(reinterpret_cast<uint32_t*>(tmp.data()), static_cast<size_t>(CoreState::kW) * CoreState::kH);

            for (;;) {
                ULONGLONG now = GetTickCount64();
                if (CoreState::g_stop || now >= nextDue) {
                    if (now > nextDue + 250) nextDue = now;
                    break;
                }
                Sleep(static_cast<DWORD>(std::min<ULONGLONG>(nextDue - now, 4)));
            }
            if (CoreState::g_stop) break;

            EnterCriticalSection(&CoreState::g_cs);
            memcpy(CoreState::g_shared.data(), tmp.data(), CoreState::g_frameBytes);
            CoreState::g_seq++;
            LeaveCriticalSection(&CoreState::g_cs);

            if (CoreState::g_wnd) {
                PostMessageW(CoreState::g_wnd, WM_NULL, 0, 0);
            }

            if (!firstPublished) {
                firstPublished = true;
                s_decPublished = true;
                CoreState::g_frameDone = true;
                std::wstring cachePath = CacheManager::GetCacheFilePath(s_mediaPath, CoreState::kW, CoreState::kH, cfg.fitMode);
                CacheManager::SaveFirstFrame(cachePath, rawFirst.data());
                Logger::Log(L"Decoder: GIF active");
            }

            UINT d = delay <= 1 ? 10 : delay;
            nextDue += static_cast<ULONGLONG>(d) * 10ULL * 100ULL / static_cast<ULONGLONG>(cfg.speed > 0 ? cfg.speed : 100);

            if (disp == 2) {
                for (UINT y = 0; y < fh && top + y < lh; y++)
                    for (UINT x = 0; x < fw && left + x < lw; x++)
                        canvas[static_cast<size_t>(top + y) * lw + (left + x)] = 0;
            } else if (disp == 3 && !prev.empty()) {
                canvas = prev;
            }

            if (++idx >= count) {
                idx = 0;
                std::fill(canvas.begin(), canvas.end(), 0u);
                CoreState::g_loopSerial++;
            }
        }

        SafeRelease(dec);
        SafeRelease(fac);
        CoreState::g_decoderDone = true;
        return 0;
    }

    static DWORD WINAPI DecoderThreadProc(LPVOID) {
        try {
            if (s_isGif) return GifRun();
            const auto& cfg = ConfigManager::Current();
            DWORD r = DecodeRun(cfg.hwDecode);
            if (cfg.hwDecode && !s_decPublished && !CoreState::g_stop) {
                Logger::Log(L"Retrying with software decoding fallback...");
                r = DecodeRun(false);
            }
            return r;
        } catch (...) {
            Logger::Log(L"Decoder thread unhandled exception");
            CoreState::g_decodeDead = true;
            return 1;
        }
    }

    bool Start(const std::wstring& mediaPath) {
        s_mediaPath = mediaPath;
        s_decPublished = false;

        size_t dot = s_mediaPath.find_last_of(L'.');
        std::wstring ext = (dot == std::wstring::npos) ? L"" : s_mediaPath.substr(dot + 1);
        s_isGif = (_wcsicmp(ext.c_str(), L"gif") == 0);

        s_decThread = CreateThread(nullptr, 0, DecoderThreadProc, nullptr, 0, nullptr);
        return s_decThread != nullptr;
    }

    void Stop() {
        if (s_decThread) {
            WaitForSingleObject(s_decThread, 3000);
            CloseHandle(s_decThread);
            s_decThread = nullptr;
        }
    }
}
