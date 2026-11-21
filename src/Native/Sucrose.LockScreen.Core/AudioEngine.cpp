#include "pch.h"
#include "AudioEngine.h"
#include "Common.h"
#include "Logger.h"

#ifndef AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
#define AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM 0x80000000
#endif
#ifndef AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY
#define AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY 0x08000000
#endif

static const CLSID kClsidMMDeviceEnumerator =
    { 0xBCDE0395, 0xE52F, 0x467C, { 0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E } };

template <class T> static void SafeRelease(T*& p) {
    if (p) { p->Release(); p = nullptr; }
}

namespace AudioEngine {
    static HANDLE s_audioThread = nullptr;
    static std::wstring s_mediaPath;
    static int s_volume = 50;

    static DWORD WINAPI AudioThreadProc(LPVOID) {
        bool coOk = SUCCEEDED(CoInitializeEx(nullptr, COINIT_MULTITHREADED));
        bool mfOk = SUCCEEDED(MFStartup(MF_VERSION));

        IMFSourceReader* rd = nullptr;
        IMMDeviceEnumerator* en = nullptr;
        IMMDevice* dev = nullptr;
        IAudioClient* ac = nullptr;
        IAudioRenderClient* rc = nullptr;

        do {
            if (!mfOk) break;

            const DWORD AS = static_cast<DWORD>(MF_SOURCE_READER_FIRST_AUDIO_STREAM);
            if (FAILED(MFCreateSourceReaderFromURL(s_mediaPath.c_str(), nullptr, &rd)) || !rd) {
                Logger::Log(L"Audio: cannot open media file for audio");
                break;
            }

            rd->SetStreamSelection(static_cast<DWORD>(MF_SOURCE_READER_ALL_STREAMS), FALSE);
            IMFMediaType* nat = nullptr;
            if (FAILED(rd->SetStreamSelection(AS, TRUE)) ||
                FAILED(rd->GetNativeMediaType(AS, 0, &nat)) || !nat) {
                Logger::Log(L"Audio: media has no audio track");
                break;
            }

            UINT32 ch = MFGetAttributeUINT32(nat, MF_MT_AUDIO_NUM_CHANNELS, 2);
            UINT32 rate = MFGetAttributeUINT32(nat, MF_MT_AUDIO_SAMPLES_PER_SECOND, 48000);
            nat->Release();
            if (ch > 2) ch = 2;
            if (ch < 1) ch = 1;

            IMFMediaType* mt = nullptr;
            if (FAILED(MFCreateMediaType(&mt)) || !mt) break;
            mt->SetGUID(MF_MT_MAJOR_TYPE, MFMediaType_Audio);
            mt->SetGUID(MF_MT_SUBTYPE, MFAudioFormat_Float);
            mt->SetUINT32(MF_MT_AUDIO_NUM_CHANNELS, ch);
            mt->SetUINT32(MF_MT_AUDIO_SAMPLES_PER_SECOND, rate);
            mt->SetUINT32(MF_MT_AUDIO_BITS_PER_SAMPLE, 32);
            mt->SetUINT32(MF_MT_AUDIO_BLOCK_ALIGNMENT, ch * 4);
            mt->SetUINT32(MF_MT_AUDIO_AVG_BYTES_PER_SECOND, rate * ch * 4);
            HRESULT hr = rd->SetCurrentMediaType(AS, nullptr, mt);
            mt->Release();
            if (FAILED(hr)) {
                Logger::Log(L"Audio: Float PCM output not supported (hr=0x%08X)", static_cast<unsigned>(hr));
                break;
            }

            if (FAILED(CoCreateInstance(kClsidMMDeviceEnumerator, nullptr, CLSCTX_ALL,
                                        __uuidof(IMMDeviceEnumerator), reinterpret_cast<void**>(&en))) || !en) break;
            if (FAILED(en->GetDefaultAudioEndpoint(eRender, eConsole, &dev)) || !dev) {
                Logger::Log(L"Audio: default audio device not found");
                break;
            }
            if (FAILED(dev->Activate(__uuidof(IAudioClient), CLSCTX_ALL, nullptr,
                                     reinterpret_cast<void**>(&ac))) || !ac) break;

            WAVEFORMATEX wf = {};
            wf.wFormatTag = WAVE_FORMAT_IEEE_FLOAT;
            wf.nChannels = static_cast<WORD>(ch);
            wf.nSamplesPerSec = rate;
            wf.wBitsPerSample = 32;
            wf.nBlockAlign = static_cast<WORD>(ch * 4);
            wf.nAvgBytesPerSec = rate * wf.nBlockAlign;

            hr = ac->Initialize(AUDCLNT_SHAREMODE_SHARED,
                                AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                                2000000, 0, &wf, nullptr);
            if (FAILED(hr)) {
                Logger::Log(L"Audio: IAudioClient::Initialize failed (hr=0x%08X)", static_cast<unsigned>(hr));
                break;
            }

            UINT32 bufFrames = 0;
            ac->GetBufferSize(&bufFrames);
            if (FAILED(ac->GetService(__uuidof(IAudioRenderClient), reinterpret_cast<void**>(&rc))) || !rc) break;

            Logger::Log(L"Audio engine started: %u channels, %u Hz", ch, rate);

            std::vector<uint8_t> pending;
            size_t pos = 0;
            bool ended = false;
            bool running = false;
            LONG seenLoop = CoreState::g_loopSerial.load();
            const UINT32 frameBytes = ch * 4;
            float gain = 0.0f;
            const float fadeStep = 1.0f / static_cast<float>(rate * 0.6); // 0.6s fade

            while (!CoreState::g_stop && !CoreState::g_decoderDone) {
                if (CoreState::g_idlePaused.load() && CoreState::g_audioDone && (!running || gain <= 0.001f)) {
                    gain = 0.0f;
                    if (running) { ac->Stop(); running = false; }
                    Sleep(100);
                    continue;
                }

                if (seenLoop != CoreState::g_loopSerial.load()) {
                    seenLoop = CoreState::g_loopSerial.load();
                    PROPVARIANT pv;
                    PropVariantInit(&pv);
                    pv.vt = VT_I8;
                    pv.hVal.QuadPart = 0;
                    rd->SetCurrentPosition(kGuidNull, pv);
                    pending.clear();
                    pos = 0;
                    ended = false;
                    ac->Stop();
                    ac->Reset();
                    gain = 0.0f;
                    running = false;
                }

                if (pos >= pending.size() && !ended) {
                    pending.clear();
                    pos = 0;
                    DWORD flags = 0;
                    IMFSample* s = nullptr;
                    hr = rd->ReadSample(AS, 0, nullptr, &flags, nullptr, &s);
                    if (FAILED(hr) || (flags & MF_SOURCE_READERF_ERROR)) {
                        if (s) s->Release();
                        break;
                    }
                    if (flags & MF_SOURCE_READERF_ENDOFSTREAM) ended = true;
                    if (s) {
                        IMFMediaBuffer* b = nullptr;
                        if (SUCCEEDED(s->ConvertToContiguousBuffer(&b)) && b) {
                            BYTE* d = nullptr;
                            DWORD cur = 0;
                            if (SUCCEEDED(b->Lock(&d, nullptr, &cur)) && d) {
                                pending.assign(d, d + cur);
                                CoreState::g_audioDone = true;
                                b->Unlock();
                            }
                            b->Release();
                        }
                        s->Release();
                    }
                }

                if (CoreState::g_idlePaused.load() && gain <= 0.001f) {
                    Sleep(10);
                    continue;
                }

                UINT32 padding = 0;
                if (FAILED(ac->GetCurrentPadding(&padding))) break;
                UINT32 freeFrames = bufFrames > padding ? bufFrames - padding : 0;
                UINT32 haveFrames = static_cast<UINT32>((pending.size() - pos) / frameBytes);
                if (haveFrames == 0) pos = pending.size();

                UINT32 n = std::min(freeFrames, haveFrames);
                if (n > 0) {
                    BYTE* outBuf = nullptr;
                    if (FAILED(rc->GetBuffer(n, &outBuf)) || !outBuf) break;

                    float vol = static_cast<float>(s_volume) / 100.0f;
                    vol *= vol; // quadratic curve for smoother low volume control

                    const float* in = reinterpret_cast<const float*>(pending.data() + pos);
                    float* o = reinterpret_cast<float*>(outBuf);
                    const float tgt = CoreState::g_idlePaused.load() ? 0.0f : 1.0f;

                    for (UINT32 i = 0; i < n; i++) {
                        if (gain < tgt) gain = std::min(tgt, gain + fadeStep);
                        else if (gain > tgt) gain = std::max(tgt, gain - fadeStep);

                        for (UINT32 c = 0; c < ch; c++) {
                            o[i * ch + c] = in[i * ch + c] * vol * gain;
                        }
                    }
                    rc->ReleaseBuffer(n, 0);
                    CoreState::g_audioDone = true;
                    pos += static_cast<size_t>(n) * frameBytes;
                    if (!running) { ac->Start(); running = true; }
                }
                Sleep(10);
            }
        } while (false);

        if (ac) ac->Stop();
        SafeRelease(rc);
        SafeRelease(ac);
        SafeRelease(dev);
        SafeRelease(en);
        SafeRelease(rd);

        if (mfOk) MFShutdown();
        if (coOk) CoUninitialize();
        CoreState::g_audioDone = true;
        Logger::Log(L"Audio thread stopped");
        return 0;
    }

    bool Start(const std::wstring& mediaPath, int volume) {
        if (volume <= 0 || mediaPath.empty()) {
            CoreState::g_audioDone = true;
            return false;
        }

        s_mediaPath = mediaPath;
        s_volume = volume;
        CoreState::g_audioDone = false;

        s_audioThread = CreateThread(nullptr, 0, AudioThreadProc, nullptr, 0, nullptr);
        return s_audioThread != nullptr;
    }

    void Stop() {
        if (s_audioThread) {
            WaitForSingleObject(s_audioThread, 3000);
            CloseHandle(s_audioThread);
            s_audioThread = nullptr;
        }
    }
}
