#include "pch.h"
#include "CredentialProvider.h"
#include "Logger.h"

// ---------------------------------------------------------
// SucroseCredentialProvider Implementation
// ---------------------------------------------------------

SucroseCredentialProvider::SucroseCredentialProvider()
    : m_cRef(1)
{
    Logger::Log(L"SucroseCredentialProvider instance created.");
}

SucroseCredentialProvider::~SucroseCredentialProvider()
{
    Logger::Log(L"SucroseCredentialProvider instance destroyed.");
}

IFACEMETHODIMP SucroseCredentialProvider::QueryInterface(REFIID riid, void** ppv)
{
    if (!ppv) return E_POINTER;
    *ppv = nullptr;

    if (riid == IID_IUnknown || riid == IID_ICredentialProvider)
    {
        *ppv = static_cast<ICredentialProvider*>(this);
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
}

IFACEMETHODIMP_(ULONG) SucroseCredentialProvider::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

IFACEMETHODIMP_(ULONG) SucroseCredentialProvider::Release()
{
    LONG count = InterlockedDecrement(&m_cRef);
    if (count == 0)
    {
        delete this;
    }
    return count;
}

IFACEMETHODIMP SucroseCredentialProvider::SetUsageScenario(
    CREDENTIAL_PROVIDER_USAGE_SCENARIO cpus,
    DWORD dwFlags)
{
    Logger::Log(L"SucroseCredentialProvider::SetUsageScenario (Scenario: %d, Flags: 0x%08X)", (int)cpus, dwFlags);
    return S_OK;
}

IFACEMETHODIMP SucroseCredentialProvider::SetSerialization(
    const CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION* pcpcs)
{
    UNREFERENCED_PARAMETER(pcpcs);
    return E_NOTIMPL;
}

IFACEMETHODIMP SucroseCredentialProvider::Advise(
    ICredentialProviderEvents* pcpe,
    UINT_PTR upAdviseContext)
{
    UNREFERENCED_PARAMETER(pcpe);
    UNREFERENCED_PARAMETER(upAdviseContext);
    return S_OK;
}

IFACEMETHODIMP SucroseCredentialProvider::UnAdvise()
{
    return S_OK;
}

IFACEMETHODIMP SucroseCredentialProvider::GetFieldDescriptorCount(DWORD* pdwCount)
{
    if (!pdwCount) return E_POINTER;
    *pdwCount = 0;
    return S_OK;
}

IFACEMETHODIMP SucroseCredentialProvider::GetFieldDescriptorAt(
    DWORD dwIndex,
    CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR** ppcpfd)
{
    UNREFERENCED_PARAMETER(dwIndex);
    UNREFERENCED_PARAMETER(ppcpfd);
    return E_INVALIDARG;
}

IFACEMETHODIMP SucroseCredentialProvider::GetCredentialCount(
    DWORD* pdwCount,
    DWORD* pdwDefault,
    BOOL* pbAutoLogonWithDefault)
{
    if (!pdwCount || !pdwDefault || !pbAutoLogonWithDefault)
    {
        return E_POINTER;
    }

    // We provide 0 credentials so we do not add any tiles or UI clutter to LogonUI
    *pdwCount = 0;
    *pdwDefault = CREDENTIAL_PROVIDER_NO_DEFAULT;
    *pbAutoLogonWithDefault = FALSE;
    return S_OK;
}

IFACEMETHODIMP SucroseCredentialProvider::GetCredentialAt(
    DWORD dwIndex,
    ICredentialProviderCredential** ppcpc)
{
    UNREFERENCED_PARAMETER(dwIndex);
    UNREFERENCED_PARAMETER(ppcpc);
    return E_INVALIDARG;
}

// ---------------------------------------------------------
// SucroseClassFactory Implementation
// ---------------------------------------------------------

SucroseClassFactory::SucroseClassFactory()
    : m_cRef(1)
{
}

SucroseClassFactory::~SucroseClassFactory()
{
}

IFACEMETHODIMP SucroseClassFactory::QueryInterface(REFIID riid, void** ppv)
{
    if (!ppv) return E_POINTER;
    *ppv = nullptr;

    if (riid == IID_IUnknown || riid == IID_IClassFactory)
    {
        *ppv = static_cast<IClassFactory*>(this);
        AddRef();
        return S_OK;
    }

    return E_NOINTERFACE;
}

IFACEMETHODIMP_(ULONG) SucroseClassFactory::AddRef()
{
    return InterlockedIncrement(&m_cRef);
}

IFACEMETHODIMP_(ULONG) SucroseClassFactory::Release()
{
    LONG count = InterlockedDecrement(&m_cRef);
    if (count == 0)
    {
        delete this;
    }
    return count;
}

IFACEMETHODIMP SucroseClassFactory::CreateInstance(
    IUnknown* pUnkOuter,
    REFIID riid,
    void** ppv)
{
    if (pUnkOuter) return CLASS_E_NOAGGREGATION;
    if (!ppv) return E_POINTER;
    *ppv = nullptr;

    SucroseCredentialProvider* pProvider = new (std::nothrow) SucroseCredentialProvider();
    if (!pProvider) return E_OUTOFMEMORY;

    HRESULT hr = pProvider->QueryInterface(riid, ppv);
    pProvider->Release();
    return hr;
}

IFACEMETHODIMP SucroseClassFactory::LockServer(BOOL fLock)
{
    UNREFERENCED_PARAMETER(fLock);
    return S_OK;
}

// ---------------------------------------------------------
// COM DLL Exports
// ---------------------------------------------------------

STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, LPVOID* ppv)
{
    if (!ppv) return E_POINTER;
    *ppv = nullptr;

    if (IsEqualCLSID(rclsid, CLSID_SucroseLockScreenProvider))
    {
        SucroseClassFactory* pFactory = new (std::nothrow) SucroseClassFactory();
        if (!pFactory) return E_OUTOFMEMORY;

        HRESULT hr = pFactory->QueryInterface(riid, ppv);
        pFactory->Release();
        return hr;
    }

    return CLASS_E_CLASSNOTAVAILABLE;
}

STDAPI DllCanUnloadNow()
{
    // Keep DLL resident in LogonUI.exe
    return S_FALSE;
}

static HRESULT CreateRegKeyAndSetValue(HKEY hRoot, const wchar_t* subKey, const wchar_t* valueName, const wchar_t* data)
{
    HKEY hKey = NULL;
    LONG res = RegCreateKeyExW(hRoot, subKey, 0, NULL, REG_OPTION_NON_VOLATILE, KEY_WRITE, NULL, &hKey, NULL);
    if (res != ERROR_SUCCESS) return HRESULT_FROM_WIN32(res);

    if (data)
    {
        res = RegSetValueExW(hKey, valueName, 0, REG_SZ, (const BYTE*)data, (DWORD)((wcslen(data) + 1) * sizeof(wchar_t)));
    }
    RegCloseKey(hKey);
    return HRESULT_FROM_WIN32(res);
}

STDAPI DllRegisterServer()
{
    wchar_t modulePath[MAX_PATH] = {};
    if (GetModuleFileNameW(g_hModule, modulePath, MAX_PATH) == 0)
    {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    // 1. HKLM\SOFTWARE\Classes\CLSID\{7C4B13B8-439D-4F18-A7F0-18DCEE32A001}
    std::wstring clsidKey = std::wstring(L"SOFTWARE\\Classes\\CLSID\\") + s_szProviderCLSID;
    HRESULT hr = CreateRegKeyAndSetValue(HKEY_LOCAL_MACHINE, clsidKey.c_str(), NULL, s_szProviderName);
    if (FAILED(hr)) return hr;

    // 2. InprocServer32 -> modulePath, ThreadingModel = "Apartment"
    std::wstring inprocKey = clsidKey + L"\\InprocServer32";
    hr = CreateRegKeyAndSetValue(HKEY_LOCAL_MACHINE, inprocKey.c_str(), NULL, modulePath);
    if (FAILED(hr)) return hr;
    hr = CreateRegKeyAndSetValue(HKEY_LOCAL_MACHINE, inprocKey.c_str(), L"ThreadingModel", L"Apartment");
    if (FAILED(hr)) return hr;

    // 3. HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\Credential Providers\{7C4B13B8-439D-4F18-A7F0-18DCEE32A001}
    std::wstring cpKey = std::wstring(L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\Credential Providers\\") + s_szProviderCLSID;
    hr = CreateRegKeyAndSetValue(HKEY_LOCAL_MACHINE, cpKey.c_str(), NULL, s_szProviderName);
    return hr;
}

STDAPI DllUnregisterServer()
{
    // 1. Delete Credential Providers key
    std::wstring cpKey = std::wstring(L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Authentication\\Credential Providers\\") + s_szProviderCLSID;
    RegDeleteKeyW(HKEY_LOCAL_MACHINE, cpKey.c_str());

    // 2. Delete CLSID InprocServer32 and CLSID key
    std::wstring clsidKey = std::wstring(L"SOFTWARE\\Classes\\CLSID\\") + s_szProviderCLSID;
    std::wstring inprocKey = clsidKey + L"\\InprocServer32";
    RegDeleteKeyW(HKEY_LOCAL_MACHINE, inprocKey.c_str());
    RegDeleteKeyW(HKEY_LOCAL_MACHINE, clsidKey.c_str());

    return S_OK;
}
