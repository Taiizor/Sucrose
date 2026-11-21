#pragma once

#include <credentialprovider.h>
#include <atomic>
#include <string>

// CLSID: {7C4B13B8-439D-4F18-A7F0-18DCEE32A001}
static const GUID CLSID_SucroseLockScreenProvider = 
    { 0x7c4b13b8, 0x439d, 0x4f18, { 0xa7, 0xf0, 0x18, 0xdc, 0xee, 0x32, 0xa0, 0x01 } };

static const wchar_t* const s_szProviderCLSID = L"{7C4B13B8-439D-4F18-A7F0-18DCEE32A001}";
static const wchar_t* const s_szProviderName = L"Sucrose Lock Screen Live Wallpaper Provider";

extern HMODULE g_hModule;

class SucroseCredentialProvider : public ICredentialProvider
{
public:
    SucroseCredentialProvider();
    virtual ~SucroseCredentialProvider();

    // IUnknown
    IFACEMETHODIMP QueryInterface(REFIID riid, void** ppv) override;
    IFACEMETHODIMP_(ULONG) AddRef() override;
    IFACEMETHODIMP_(ULONG) Release() override;

    // ICredentialProvider
    IFACEMETHODIMP SetUsageScenario(CREDENTIAL_PROVIDER_USAGE_SCENARIO cpus, DWORD dwFlags) override;
    IFACEMETHODIMP SetSerialization(const CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION* pcpcs) override;
    IFACEMETHODIMP Advise(ICredentialProviderEvents* pcpe, UINT_PTR upAdviseContext) override;
    IFACEMETHODIMP UnAdvise() override;
    IFACEMETHODIMP GetFieldDescriptorCount(DWORD* pdwCount) override;
    IFACEMETHODIMP GetFieldDescriptorAt(DWORD dwIndex, CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR** ppcpfd) override;
    IFACEMETHODIMP GetCredentialCount(DWORD* pdwCount, DWORD* pdwDefault, BOOL* pbAutoLogonWithDefault) override;
    IFACEMETHODIMP GetCredentialAt(DWORD dwIndex, ICredentialProviderCredential** ppcpc) override;

private:
    LONG m_cRef;
};

class SucroseClassFactory : public IClassFactory
{
public:
    SucroseClassFactory();
    virtual ~SucroseClassFactory();

    // IUnknown
    IFACEMETHODIMP QueryInterface(REFIID riid, void** ppv) override;
    IFACEMETHODIMP_(ULONG) AddRef() override;
    IFACEMETHODIMP_(ULONG) Release() override;

    // IClassFactory
    IFACEMETHODIMP CreateInstance(IUnknown* pUnkOuter, REFIID riid, void** ppv) override;
    IFACEMETHODIMP LockServer(BOOL fLock) override;

private:
    LONG m_cRef;
};
