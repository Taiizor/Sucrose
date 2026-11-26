//! Authenticode verification of a downloaded installer (D-15, D-019, T-04-41). The file is accepted only when
//! WinVerifyTrust accepts its signature, the leaf signing certificate's common name is exactly `Microsoft Corporation`
//! and the chain ends in a Microsoft root (the OS's `CERT_CHAIN_POLICY_MICROSOFT_ROOT` policy). Nothing is written and
//! no window is shown.
//!
//! Revocation: the whole chain is checked online first. When the OS cannot reach a revocation source (offline, no CRL
//! point) the same verification is repeated without the revocation check, because the signature, digest, signer and
//! Microsoft root are still checked in full and a download needed a connection a moment ago; a certificate the OS
//! knows to be revoked is always refused.

use std::{
    ffi::c_void,
    fs::File,
    os::windows::{ffi::OsStrExt, io::AsRawHandle},
    path::Path,
    ptr,
};

use windows::{
    Win32::{
        Foundation::{HANDLE, HWND},
        Security::{
            Cryptography::{
                CERT_CHAIN_POLICY_FLAGS, CERT_CHAIN_POLICY_MICROSOFT_ROOT, CERT_CHAIN_POLICY_PARA,
                CERT_CHAIN_POLICY_STATUS, CERT_NAME_ATTR_TYPE, CertGetNameStringW, CertVerifyCertificateChainPolicy,
            },
            WinTrust::{
                WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_DATA_REVOCATION_CHECKS,
                WINTRUST_FILE_INFO, WTD_CHOICE_FILE, WTD_REVOKE_NONE, WTD_REVOKE_WHOLECHAIN, WTD_STATEACTION_CLOSE,
                WTD_STATEACTION_VERIFY, WTD_UI_NONE, WTHelperGetProvCertFromChain, WTHelperGetProvSignerFromChain,
                WTHelperProvDataFromStateData, WinVerifyTrust,
            },
        },
    },
    core::{GUID, PCSTR, PCWSTR},
};

use crate::{SignerInfo, TrustError};

/// The only signer name an installer may have.
const MICROSOFT_SIGNER: &str = "Microsoft Corporation";

/// The object identifier of the common name attribute (`szOID_COMMON_NAME`), NUL-terminated for the Win32 call.
const COMMON_NAME_OID: &[u8] = b"2.5.4.3\0";

/// `MICROSOFT_ROOT_CERT_CHAIN_POLICY_CHECK_APPLICATION_ROOT_FLAG`: the chain must end in a Microsoft root used for application
/// code. Without it the policy only accepts the roots of the OS itself and refuses the root of the Visual C++
/// bootstrapper (Microsoft Root Certificate Authority 2011, CERT_E_UNTRUSTEDROOT, measured in plan 04-18).
const MICROSOFT_ROOT_CHECK_APPLICATION_ROOT: CERT_CHAIN_POLICY_FLAGS = CERT_CHAIN_POLICY_FLAGS(0x0002_0000);

/// WinVerifyTrust codes this module tells apart (HRESULT-style values, written as the unsigned numbers the SDK lists).
const TRUST_E_PROVIDER_UNKNOWN: u32 = 0x800B_0001;
const TRUST_E_SUBJECT_FORM_UNKNOWN: u32 = 0x800B_0003;
const TRUST_E_NOSIGNATURE: u32 = 0x800B_0100;
const TRUST_E_BAD_DIGEST: u32 = 0x8009_6010;
const TRUST_E_CERT_SIGNATURE: u32 = 0x8009_6004;
const TRUST_E_COUNTER_SIGNER: u32 = 0x8009_6003;
const CERT_E_REVOKED: u32 = 0x800B_010C;
const CRYPT_E_REVOKED: u32 = 0x8009_2010;
const CRYPT_E_NO_REVOCATION_CHECK: u32 = 0x8009_2012;
const CRYPT_E_REVOCATION_OFFLINE: u32 = 0x8009_2013;

/// What a nonzero WinVerifyTrust answer means for this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    NotSigned,
    BadSignature,
    Revoked,
    /// The OS could not look up revocation; the check is repeated without it.
    RevocationUnknown,
    Untrusted,
}

/// Classifies a nonzero WinVerifyTrust answer. Anything not listed is `Untrusted`: the check fails closed.
fn classify(status: i32) -> Verdict {
    #[allow(clippy::cast_sign_loss, reason = "the SDK lists these HRESULTs as unsigned numbers")]
    match status as u32 {
        TRUST_E_NOSIGNATURE | TRUST_E_SUBJECT_FORM_UNKNOWN | TRUST_E_PROVIDER_UNKNOWN => Verdict::NotSigned,
        TRUST_E_BAD_DIGEST | TRUST_E_CERT_SIGNATURE | TRUST_E_COUNTER_SIGNER => Verdict::BadSignature,
        CERT_E_REVOKED | CRYPT_E_REVOKED => Verdict::Revoked,
        CRYPT_E_NO_REVOCATION_CHECK | CRYPT_E_REVOCATION_OFFLINE => Verdict::RevocationUnknown,
        _ => Verdict::Untrusted,
    }
}

/// Whether `name` is exactly the one signer Sucrose runs installers from.
fn is_microsoft_signer(name: &str) -> bool {
    name == MICROSOFT_SIGNER
}

/// Closes the WinVerifyTrust state when dropped, on every path out of a verification.
struct StateClose {
    data: *mut WINTRUST_DATA,
}

impl Drop for StateClose {
    fn drop(&mut self) {
        let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
        // SAFETY: `data` points to the WINTRUST_DATA the verify call filled; it outlives this guard (the guard is
        // declared after it), and the CLOSE action only releases the state that call allocated.
        unsafe {
            (*self.data).dwStateAction = WTD_STATEACTION_CLOSE;
            let _ = WinVerifyTrust(no_window(), &raw mut action, self.data.cast::<c_void>());
        }
    }
}

/// The handle that tells WinVerifyTrust to show no window.
fn no_window() -> HWND {
    HWND(-1_isize as *mut c_void)
}

/// What one WinVerifyTrust run found.
enum Run {
    Accepted(SignerInfo),
    Refused(TrustError),
    /// The OS could not look up revocation.
    RevocationUnknown,
}

/// Runs WinVerifyTrust on the open `file` (named `wide`) and, when it accepts, reads the signer and checks the chain.
fn run(file: &File, wide: &[u16], revocation: WINTRUST_DATA_REVOCATION_CHECKS) -> Run {
    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: PCWSTR(wide.as_ptr()),
        hFile: HANDLE(file.as_raw_handle()),
        pgKnownSubject: ptr::null_mut(),
    };
    let mut data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: revocation,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 {
            pFile: &raw mut file_info,
        },
        dwStateAction: WTD_STATEACTION_VERIFY,
        ..WINTRUST_DATA::default()
    };
    let data_ptr: *mut WINTRUST_DATA = &raw mut data;
    let mut action: GUID = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    // SAFETY: `data` is fully initialised, its file info and the NUL-terminated path in `wide` live until after the
    // matching CLOSE call below, and the handle belongs to `file`, which the caller keeps open.
    let status = unsafe { WinVerifyTrust(no_window(), &raw mut action, data_ptr.cast::<c_void>()) };
    let _close = StateClose { data: data_ptr };
    if status != 0 {
        return match classify(status) {
            Verdict::NotSigned => Run::Refused(TrustError::NotSigned),
            Verdict::BadSignature => Run::Refused(TrustError::BadSignature),
            Verdict::Revoked => Run::Refused(TrustError::Revoked),
            Verdict::RevocationUnknown => Run::RevocationUnknown,
            Verdict::Untrusted => Run::Refused(TrustError::Untrusted { code: status }),
        };
    }
    // SAFETY: the verify call succeeded, so `hWVTStateData` is the state it allocated; it stays valid until `_close` drops.
    match unsafe { signer_of(data_ptr) } {
        Ok(signer) => Run::Accepted(signer),
        Err(error) => Run::Refused(error),
    }
}

/// Reads the leaf signer of the first signature in the verification state and checks the Microsoft root policy.
///
/// # Safety
/// `data` must point to a WINTRUST_DATA whose verification just succeeded and whose state has not been closed.
unsafe fn signer_of(data: *mut WINTRUST_DATA) -> Result<SignerInfo, TrustError> {
    // SAFETY: per the function contract `data` is valid and holds the state handle; the helpers return null for what
    // they cannot find, which is checked before each use, and the returned structures belong to the state.
    unsafe {
        let provider = WTHelperProvDataFromStateData((*data).hWVTStateData);
        if provider.is_null() {
            return Err(TrustError::Os {
                context: "reading the signature state",
                code: 0,
            });
        }
        let signer = WTHelperGetProvSignerFromChain(provider, 0, false, 0);
        if signer.is_null() {
            return Err(TrustError::Os {
                context: "reading the signer",
                code: 0,
            });
        }
        let leaf = WTHelperGetProvCertFromChain(signer, 0);
        if leaf.is_null() || (*leaf).pCert.is_null() {
            return Err(TrustError::Os {
                context: "reading the signing certificate",
                code: 0,
            });
        }
        let mut name = [0_u16; 256];
        let oid = PCSTR(COMMON_NAME_OID.as_ptr());
        let written = CertGetNameStringW(
            (*leaf).pCert,
            CERT_NAME_ATTR_TYPE,
            0,
            Some(oid.0.cast::<c_void>()),
            Some(&mut name),
        );
        let length = (written as usize).saturating_sub(1).min(name.len());
        let subject = String::from_utf16_lossy(&name[..length]);
        if !is_microsoft_signer(&subject) {
            return Err(TrustError::NotMicrosoft { signer: subject });
        }
        let chain = (*signer).pChainContext;
        if chain.is_null() {
            return Err(TrustError::NotMicrosoft { signer: subject });
        }
        let para = CERT_CHAIN_POLICY_PARA {
            cbSize: size_of::<CERT_CHAIN_POLICY_PARA>() as u32,
            dwFlags: MICROSOFT_ROOT_CHECK_APPLICATION_ROOT,
            ..CERT_CHAIN_POLICY_PARA::default()
        };
        let mut status = CERT_CHAIN_POLICY_STATUS {
            cbSize: size_of::<CERT_CHAIN_POLICY_STATUS>() as u32,
            ..CERT_CHAIN_POLICY_STATUS::default()
        };
        let ok = CertVerifyCertificateChainPolicy(
            CERT_CHAIN_POLICY_MICROSOFT_ROOT,
            chain,
            &raw const para,
            &raw mut status,
        );
        if !ok.as_bool() || status.dwError != 0 {
            return Err(TrustError::NotMicrosoft { signer: subject });
        }
        Ok(SignerInfo { subject })
    }
}

/// See [`crate::Platform::verify_microsoft_signature`].
pub(crate) fn verify_microsoft_signature(path: &Path) -> Result<SignerInfo, TrustError> {
    let file = File::open(path).map_err(|error| TrustError::Io(error.to_string()))?;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    match run(&file, &wide, WTD_REVOKE_WHOLECHAIN) {
        Run::Accepted(signer) => Ok(signer),
        Run::Refused(error) => Err(error),
        Run::RevocationUnknown => match run(&file, &wide, WTD_REVOKE_NONE) {
            Run::Accepted(signer) => Ok(signer),
            Run::Refused(error) => Err(error),
            Run::RevocationUnknown => Err(TrustError::Untrusted {
                code: CRYPT_E_NO_REVOCATION_CHECK as i32,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_exact_microsoft_name_passes() {
        assert!(is_microsoft_signer("Microsoft Corporation"));
        assert!(!is_microsoft_signer("Microsoft Corporation "));
        assert!(!is_microsoft_signer("microsoft corporation"));
        assert!(!is_microsoft_signer("Microsoft Corp"));
        assert!(!is_microsoft_signer("Evil Microsoft Corporation"));
        assert!(!is_microsoft_signer(""));
    }

    #[test]
    fn os_verdicts_are_told_apart_and_unknown_ones_fail_closed() {
        assert_eq!(classify(TRUST_E_NOSIGNATURE as i32), Verdict::NotSigned);
        assert_eq!(classify(TRUST_E_SUBJECT_FORM_UNKNOWN as i32), Verdict::NotSigned);
        assert_eq!(classify(TRUST_E_BAD_DIGEST as i32), Verdict::BadSignature);
        assert_eq!(classify(CERT_E_REVOKED as i32), Verdict::Revoked);
        assert_eq!(classify(CRYPT_E_REVOCATION_OFFLINE as i32), Verdict::RevocationUnknown);
        assert_eq!(classify(0x800B_0109_u32 as i32), Verdict::Untrusted);
        assert_eq!(classify(1), Verdict::Untrusted);
    }
}
