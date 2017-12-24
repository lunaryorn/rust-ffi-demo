// Copyright 2017 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

//! Provide acccess to the macOS Keychain.

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod native;
mod cfutil;

use std;
use std::fmt;
use std::ptr;
use std::os::raw::c_void;

use self::native::*;
use self::cfutil::*;

/// A keychain error code.
#[derive(PartialEq, Debug)]
pub enum KeychainErrorCode {
    /// Authorization and/or authentication failed.
    AuthFailed,
    /// The item already exists.
    DuplicateItem,
    /// The item cannot be found.
    ItemNotFound,
    /// An invalid attempt to change the owner of an item
    InvalidOwnerEdit,
    /// An unknown keychain error code.
    ///
    /// This API wraps only common status codes; uncommon status codes end up
    /// in this variant.
    UnknownStatusCode(i32),
}

impl From<OSStatus> for KeychainErrorCode {
    fn from(status: OSStatus) -> KeychainErrorCode {
        if status == errSecAuthFailed {
            KeychainErrorCode::AuthFailed
        } else if status == errSecDuplicateItem {
            KeychainErrorCode::DuplicateItem
        } else if status == errSecItemNotFound {
            KeychainErrorCode::ItemNotFound
        } else if status == errSecInvalidOwnerEdit {
            KeychainErrorCode::InvalidOwnerEdit
        } else {
            KeychainErrorCode::UnknownStatusCode(status)
        }
    }
}

/// A keychain error.
#[derive(Debug)]
pub struct KeychainError {
    /// The status code of the error.
    ///
    /// Use this code to unambiguously identify the cause of an error.
    pub status: KeychainErrorCode,
    /// A human-readable, non-localized message for the error.
    pub message: String,
}

impl From<OSStatus> for KeychainError {
    /// Creates a `KeychainError` from an `OSStatus` value.
    ///
    /// Gets the error message from the system.
    fn from(status: OSStatus) -> KeychainError {
        let message = unsafe {
            let cf_message = SecCopyErrorMessageString(status, ptr::null_mut());
            let s = string_from_cf_string(cf_message);
            CFRelease(cf_message as CFTypeRef);
            s
        };
        KeychainError {
            status: status.into(),
            message,
        }
    }
}

impl fmt::Display for KeychainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Keychain error: {} (status: {:?})",
            self.message,
            self.status
        )
    }
}

/// An account, with an account, eg, user `name` and a `password`.
#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub password: String,
}

/// The Result of a keychain operation.
pub type Result<T> = std::result::Result<T, KeychainError>;

/// Create a result from a `status`.
///
/// If `status` is `errSecSuccess` return Ok of unit, otherwise return `Err`
/// with the corresponding `KeychainError`.
fn status_to_result(status: OSStatus) -> Result<()> {
    if status == errSecSuccess {
        Ok(())
    } else {
        Err(status.into())
    }
}

/// Add a generic account.
///
/// The `service` identifies the application or service for which the `account`
/// is being stored.
///
/// # Errors
///
/// Return `KeychainError` when the combination of `service` and `account.name`
/// already exist in keychain, or keychain access fails otherwise.
pub fn add_generic_password(service: &str, account: &Account) -> Result<()> {
    unsafe {
        let cf_service = CFStringCreateWithBytesNoCopy(
            std::ptr::null_mut(),
            service.as_ptr(),
            service.len() as i64,
            kCFStringEncodingUTF8,
            false as u8,
            kCFAllocatorNull,
        ) as CFTypeRef;
        assert!(!cf_service.is_null());
        let cf_account = CFStringCreateWithBytesNoCopy(
            ptr::null_mut(),
            account.name.as_ptr(),
            account.name.len() as i64,
            kCFStringEncodingUTF8,
            false as u8,
            kCFAllocatorNull,
        ) as CFTypeRef;
        assert!(!cf_account.is_null());
        let cf_password = CFDataCreateWithBytesNoCopy(
            ptr::null_mut(),
            account.password.as_ptr(),
            account.password.len() as i64,
            kCFAllocatorNull,
        ) as CFTypeRef;
        assert!(!cf_password.is_null());

        let items = [
            (
                kSecClass as CFTypeRef,
                kSecClassGenericPassword as CFTypeRef,
            ),
            (kSecAttrService as CFTypeRef, cf_service),
            (kSecAttrAccount as CFTypeRef, cf_account),
            (kSecValueData as CFTypeRef, cf_password),
        ];
        let attributes = create_dictionary(&items);
        assert!(!attributes.is_null());

        let status = SecItemAdd(attributes, ptr::null_mut());

        CFRelease(attributes as CFTypeRef);
        CFRelease(cf_service);
        CFRelease(cf_account);
        CFRelease(cf_password);

        status_to_result(status)
    }
}

/// Delete all generic passwords from keychain matching the given `service`.
///
/// # Errors
///
/// This function should not fail unless keychain unlocking fails.
pub fn delete_generic_passwords_by_service(service: &str) -> Result<()> {
    unsafe {
        let cf_service = CFStringCreateWithBytesNoCopy(
            std::ptr::null_mut(),
            service.as_ptr(),
            service.len() as i64,
            kCFStringEncodingUTF8,
            false as u8,
            kCFAllocatorNull,
        ) as CFTypeRef;
        assert!(!cf_service.is_null());

        let items = [
            (
                kSecClass as CFTypeRef,
                kSecClassGenericPassword as CFTypeRef,
            ),
            (kSecAttrService as CFTypeRef, cf_service),
        ];
        let query = create_dictionary(&items);
        assert!(!query.is_null());

        let status = SecItemDelete(query);

        CFRelease(query as CFTypeRef);
        CFRelease(cf_service);

        status_to_result(status)
    }
}

/// Find the first generic password for the given `service`.
///
/// # Errors
///
/// Return `KeychainError` when the item does not exist, or keychain access
/// fails otherwise.
pub fn find_generic_password_by_service(service: &str) -> Result<Account> {
    unsafe {
        let cf_service = CFStringCreateWithBytesNoCopy(
            std::ptr::null_mut(),
            service.as_ptr(),
            service.len() as i64,
            kCFStringEncodingUTF8,
            false as u8,
            kCFAllocatorNull,
        ) as CFTypeRef;
        assert!(!cf_service.is_null());

        let items = [
            (
                kSecClass as CFTypeRef,
                kSecClassGenericPassword as CFTypeRef,
            ),
            (kSecAttrService as CFTypeRef, cf_service),
            (kSecMatchLimit as CFTypeRef, kSecMatchLimitOne as CFTypeRef),
            (
                kSecReturnAttributes as CFTypeRef,
                kCFBooleanTrue as CFTypeRef,
            ),
            (kSecReturnData as CFTypeRef, kCFBooleanTrue as CFTypeRef),
        ];
        let query = create_dictionary(&items);
        assert!(!query.is_null());

        let mut result: CFTypeRef = ptr::null();
        let status = SecItemCopyMatching(query, &mut result);

        CFRelease(cf_service);
        CFRelease(query as CFTypeRef);

        status_to_result(status)?;

        assert!(!result.is_null());

        let cf_account =
            CFDictionaryGetValue(result as CFDictionaryRef, kSecAttrAccount as *const c_void)
                as CFStringRef;
        let cf_password =
            CFDictionaryGetValue(result as CFDictionaryRef, kSecValueData as *const c_void)
                as CFDataRef;

        let account = Account {
            name: string_from_cf_string(cf_account),
            password: String::from_utf8_unchecked(vec_from_cfdata(cf_password)),
        };

        // As `CFDictionaryGetValue` follows the `Get` rule, ie, ownership of
        // returned values is tied to the containing dictionary, we must NOT
        // free `cf_account` and `cf_password` here!  We just free the entire
        // `result` dictionary and it’ll free everything that’s in it.
        CFRelease(result);

        Ok(account)
    }
}
