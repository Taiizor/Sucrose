#include "pch.h"
#include "Config.h"
#include "Logger.h"

static LockScreenConfig s_currentConfig;

static inline int ClampInt(int v, int lo, int hi) {
    return v < lo ? lo : (v > hi ? hi : v);
}

void LockScreenConfig::CalculateColorMatrices() {
    isLutOnly = (hue == 0 && saturation == 100);
    isIdentity = (hue == 0 && saturation == 100 && brightness == 0 &&
                  contrast == 100 && gamma == 100 && dim == 0);

    double a = hue * 3.14159265358979 / 180.0;
    double c = cos(a), s = sin(a);
    double H[3][3] = {
        { 0.213 + c * 0.787 - s * 0.213, 0.715 - c * 0.715 - s * 0.715,
          0.072 - c * 0.072 + s * 0.928 },
        { 0.213 - c * 0.213 + s * 0.143, 0.715 + c * 0.285 + s * 0.140,
          0.072 - c * 0.072 - s * 0.283 },
        { 0.213 - c * 0.213 - s * 0.787, 0.715 - c * 0.715 + s * 0.715,
          0.072 + c * 0.928 + s * 0.072 }
    };
    double lum[3] = { 0.213, 0.715, 0.072 };
    double sv = saturation / 100.0;
    double S[3][3];
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            S[i][j] = (1.0 - sv) * lum[j] + (i == j ? sv : 0.0);
        }
    }
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            double v = 0;
            for (int k = 0; k < 3; k++) v += S[i][k] * H[k][j];
            colorMatrix[i * 3 + j] = static_cast<int>(lround(v * 256.0));
        }
    }

    double contrastF = contrast / 100.0;
    double brightF = brightness / 100.0;
    double gammaF = gamma / 100.0;
    for (int i = 0; i < 256; i++) {
        double x = i / 255.0;
        x = (x - 0.5) * contrastF + 0.5 + brightF;
        if (x < 0) x = 0;
        if (x > 1) x = 1;
        x = pow(x, 1.0 / gammaF);
        x *= 1.0 - dim / 100.0;
        lut[i] = static_cast<uint8_t>(lround(x * 255.0));
    }
}

static inline uint32_t Fx(const LockScreenConfig& cfg, uint32_t p) {
    int r = (p >> 16) & 255, g = (p >> 8) & 255, b = p & 255;
    int nr = (cfg.colorMatrix[0] * r + cfg.colorMatrix[1] * g + cfg.colorMatrix[2] * b) >> 8;
    int ng = (cfg.colorMatrix[3] * r + cfg.colorMatrix[4] * g + cfg.colorMatrix[5] * b) >> 8;
    int nb = (cfg.colorMatrix[6] * r + cfg.colorMatrix[7] * g + cfg.colorMatrix[8] * b) >> 8;
    nr = nr < 0 ? 0 : (nr > 255 ? 255 : nr);
    ng = ng < 0 ? 0 : (ng > 255 ? 255 : ng);
    nb = nb < 0 ? 0 : (nb > 255 ? 255 : nb);
    return 0xFF000000u | (static_cast<uint32_t>(cfg.lut[nr]) << 16) |
           (static_cast<uint32_t>(cfg.lut[ng]) << 8) | cfg.lut[nb];
}

static inline uint32_t FxLut(const LockScreenConfig& cfg, uint32_t p) {
    return 0xFF000000u | (static_cast<uint32_t>(cfg.lut[(p >> 16) & 255]) << 16) |
           (static_cast<uint32_t>(cfg.lut[(p >> 8) & 255]) << 8) | cfg.lut[p & 255];
}

void LockScreenConfig::ApplyColor(uint32_t* pixels, size_t count) const {
    if (isIdentity) return;
    const bool keepBlack = (fitMode == kFitFit || fitMode == kFitCenter);
    for (size_t i = 0; i < count; i++) {
        uint32_t p = pixels[i];
        if (keepBlack && p == 0xFF000000u) continue;
        pixels[i] = isLutOnly ? FxLut(*this, p) : Fx(*this, p);
    }
}

namespace ConfigManager {
    LockScreenConfig& Current() {
        return s_currentConfig;
    }

    static bool IsMediaExtension(const std::wstring& filename) {
        size_t p = filename.find_last_of(L'.');
        if (p == std::wstring::npos) return false;
        std::wstring ext = filename.substr(p + 1);
        for (auto& c : ext) c = static_cast<wchar_t>(towlower(c));
        return ext == L"mp4" || ext == L"m4v" || ext == L"mov" || ext == L"wmv" ||
               ext == L"avi" || ext == L"mkv" || ext == L"gif";
    }

    std::wstring ResolveMediaFile(const std::wstring& path, int folderOrder) {
        DWORD attr = GetFileAttributesW(path.c_str());
        if (attr == INVALID_FILE_ATTRIBUTES) {
            Logger::Log(L"Media path not found: %s", path.c_str());
            return L"";
        }
        if (!(attr & FILE_ATTRIBUTE_DIRECTORY)) {
            return path;
        }

        std::vector<std::wstring> files;
        WIN32_FIND_DATAW fd;
        HANDLE h = FindFirstFileW((path + L"\\*").c_str(), &fd);
        if (h != INVALID_HANDLE_VALUE) {
            do {
                if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) && IsMediaExtension(fd.cFileName)) {
                    files.push_back(path + L"\\" + fd.cFileName);
                }
            } while (FindNextFileW(h, &fd));
            FindClose(h);
        }

        if (files.empty()) {
            Logger::Log(L"No video files found in directory: %s", path.c_str());
            return L"";
        }

        std::sort(files.begin(), files.end(), [](const std::wstring& a, const std::wstring& b) {
            return _wcsicmp(a.c_str(), b.c_str()) < 0;
        });

        if (folderOrder == 2) { // fixed
            return files[0];
        }
        if (folderOrder == 1) { // sequential
            // Simple sequential index based on local tick count mod size
            return files[(GetTickCount64() / 1000) % files.size()];
        }
        // Random
        return files[GetTickCount() % files.size()];
    }

    bool Load(const std::wstring& configFilePath) {
        // Defaults
        s_currentConfig = LockScreenConfig{};

        HANDLE hFile = CreateFileW(configFilePath.c_str(), GENERIC_READ, FILE_SHARE_READ,
                                   nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
        if (hFile == INVALID_HANDLE_VALUE) {
            Logger::Log(L"Config file not found (%s), using defaults.", configFilePath.c_str());
            s_currentConfig.CalculateColorMatrices();
            return false;
        }

        DWORD fileSize = GetFileSize(hFile, nullptr);
        if (fileSize == 0 || fileSize > 1024 * 1024) {
            CloseHandle(hFile);
            s_currentConfig.CalculateColorMatrices();
            return false;
        }

        std::string jsonUtf8(fileSize, '\0');
        DWORD bytesRead = 0;
        ReadFile(hFile, &jsonUtf8[0], fileSize, &bytesRead, nullptr);
        CloseHandle(hFile);

        int wideLen = MultiByteToWideChar(CP_UTF8, 0, jsonUtf8.c_str(), static_cast<int>(bytesRead), nullptr, 0);
        if (wideLen <= 0) {
            s_currentConfig.CalculateColorMatrices();
            return false;
        }

        std::wstring jsonWide(wideLen, L'\0');
        MultiByteToWideChar(CP_UTF8, 0, jsonUtf8.c_str(), static_cast<int>(bytesRead), &jsonWide[0], wideLen);

        try {
            winrt::Windows::Data::Json::JsonObject root;
            if (!winrt::Windows::Data::Json::JsonObject::TryParse(jsonWide, root) || !root) {
                Logger::Log(L"Failed to parse config.json syntax");
                s_currentConfig.CalculateColorMatrices();
                return false;
            }

            if (root.HasKey(L"General")) {
                auto gen = root.GetNamedObject(L"General");
                if (gen.HasKey(L"Enabled")) s_currentConfig.enabled = gen.GetNamedBoolean(L"Enabled");
                if (gen.HasKey(L"MediaPath")) s_currentConfig.mediaPath = gen.GetNamedString(L"MediaPath");
                if (gen.HasKey(L"Speed")) s_currentConfig.speed = ClampInt(static_cast<int>(gen.GetNamedNumber(L"Speed")), 25, 400);
                if (gen.HasKey(L"AudioVolume")) s_currentConfig.audioVolume = ClampInt(static_cast<int>(gen.GetNamedNumber(L"AudioVolume")), 0, 100);
                if (gen.HasKey(L"FolderOrder")) s_currentConfig.folderOrder = ClampInt(static_cast<int>(gen.GetNamedNumber(L"FolderOrder")), 0, 2);

                if (gen.HasKey(L"FitMode")) {
                    auto fm = gen.GetNamedString(L"FitMode");
                    if (_wcsicmp(fm.c_str(), L"fill") == 0) s_currentConfig.fitMode = kFitFill;
                    else if (_wcsicmp(fm.c_str(), L"fit") == 0) s_currentConfig.fitMode = kFitFit;
                    else if (_wcsicmp(fm.c_str(), L"center") == 0) s_currentConfig.fitMode = kFitCenter;
                    else s_currentConfig.fitMode = kFitCover;
                }
            }

            if (root.HasKey(L"Performance")) {
                auto perf = root.GetNamedObject(L"Performance");
                if (perf.HasKey(L"RenderWidth")) s_currentConfig.renderWidth = ClampInt(static_cast<int>(perf.GetNamedNumber(L"RenderWidth")), 320, 1920);
                if (perf.HasKey(L"HwDecode")) s_currentConfig.hwDecode = perf.GetNamedBoolean(L"HwDecode");
                if (perf.HasKey(L"PlayOnPasswordScreen")) s_currentConfig.playOnPasswordScreen = perf.GetNamedBoolean(L"PlayOnPasswordScreen");
                if (perf.HasKey(L"IdlePauseSeconds")) s_currentConfig.idlePauseSeconds = static_cast<DWORD>(perf.GetNamedNumber(L"IdlePauseSeconds"));
                if (perf.HasKey(L"LowBatteryThreshold")) s_currentConfig.lowBatteryThreshold = ClampInt(static_cast<int>(perf.GetNamedNumber(L"LowBatteryThreshold")), 0, 100);
                if (perf.HasKey(L"LowBatteryLevel")) s_currentConfig.lowBatteryLevel = ClampInt(static_cast<int>(perf.GetNamedNumber(L"LowBatteryLevel")), 0, 3);
            }

            if (root.HasKey(L"ColorAdjustments")) {
                auto col = root.GetNamedObject(L"ColorAdjustments");
                if (col.HasKey(L"Hue")) s_currentConfig.hue = ClampInt(static_cast<int>(col.GetNamedNumber(L"Hue")), -180, 180);
                if (col.HasKey(L"Saturation")) s_currentConfig.saturation = ClampInt(static_cast<int>(col.GetNamedNumber(L"Saturation")), 0, 300);
                if (col.HasKey(L"Brightness")) s_currentConfig.brightness = ClampInt(static_cast<int>(col.GetNamedNumber(L"Brightness")), -100, 100);
                if (col.HasKey(L"Contrast")) s_currentConfig.contrast = ClampInt(static_cast<int>(col.GetNamedNumber(L"Contrast")), 25, 300);
                if (col.HasKey(L"Gamma")) s_currentConfig.gamma = ClampInt(static_cast<int>(col.GetNamedNumber(L"Gamma")), 20, 300);
                if (col.HasKey(L"Dim")) s_currentConfig.dim = ClampInt(static_cast<int>(col.GetNamedNumber(L"Dim")), 0, 90);
            }

            if (root.HasKey(L"Screens")) {
                auto scr = root.GetNamedObject(L"Screens");
                if (scr.HasKey(L"ShowOnShutdown")) s_currentConfig.showOnShutdown = scr.GetNamedBoolean(L"ShowOnShutdown");
                if (scr.HasKey(L"BootStillSeconds")) s_currentConfig.bootStillSeconds = static_cast<DWORD>(scr.GetNamedNumber(L"BootStillSeconds"));
                if (scr.HasKey(L"BootStillFrame")) s_currentConfig.bootStillFrame = scr.GetNamedBoolean(L"BootStillFrame");
            }
        } catch (...) {
            Logger::Log(L"Exception occurred while parsing config.json");
        }

        s_currentConfig.CalculateColorMatrices();
        Logger::Log(L"Config loaded: Media='%s', Fit=%d, Width=%d, Volume=%d",
                    s_currentConfig.mediaPath.c_str(), s_currentConfig.fitMode,
                    s_currentConfig.renderWidth, s_currentConfig.audioVolume);
        return true;
    }
}
