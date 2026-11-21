#pragma once

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif

#ifndef NOMINMAX
#define NOMINMAX
#endif

#include <windows.h>
#include <shellapi.h>
#include <uxtheme.h>
#include <shlobj.h>
#include <shlwapi.h>
#undef GetCurrentTime
#include <timeapi.h>

// Media Foundation & Direct3D
#include <mfapi.h>
#include <mfidl.h>
#include <mfreadwrite.h>
#include <d3d11_4.h>

// Audio (WASAPI)
#include <mmreg.h>
#include <mmdeviceapi.h>
#include <audioclient.h>

// WIC (GIF)
#include <wincodec.h>

// Standard C++
#include <algorithm>
#include <atomic>
#include <cstdarg>
#include <cmath>
#include <string>
#include <vector>
#include <memory>
#include <filesystem>

// C++/WinRT
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Foundation.Collections.h>
#include <winrt/Windows.Storage.Streams.h>
#include <winrt/Windows.Data.Json.h>
#include <winrt/Windows.UI.Xaml.h>
#include <winrt/Windows.UI.Xaml.Controls.h>
#include <winrt/Windows.UI.Xaml.Media.h>
#include <winrt/Windows.UI.Xaml.Media.Imaging.h>
