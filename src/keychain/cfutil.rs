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

//! Utilities for CoreFoundation.

use std;
use super::native::*;

/// Converts a CoreFoundation String to a rust `String`.
///
/// # Safety
///
/// The caller must ensure that `cfstring` is not null.
pub unsafe fn string_from_cf_string(cfstring: CFStringRef) -> String {
    assert!(!cfstring.is_null());
    let cf_utf8 = CFStringCreateExternalRepresentation(
        std::ptr::null_mut(),
        cfstring,
        kCFStringEncodingUTF8,
        0,
    );
    let string = String::from_utf8_unchecked(vec_from_cfdata(cf_utf8));
    CFRelease(cf_utf8 as CFTypeRef);
    string
}

/// Converts a `CFData` to a vector.
///
/// # Safety
///
/// The caller must ensure that `cfdata` is not null.
pub unsafe fn vec_from_cfdata(cfdata: CFDataRef) -> Vec<u8> {
    assert!(!cfdata.is_null());
    std::slice::from_raw_parts(CFDataGetBytePtr(cfdata), CFDataGetLength(cfdata) as usize).into()
}

/// Create a `CFDictionary` from keys and values.
///
/// # Safety
///
/// `keys` and `values` must have the same length, obviously. `keys` and
/// `values` must have types as expected by whoever uses the dictionary; they
/// must also only contain CoreFoundation types!
///
/// The function does not modify `keys` and `values`, but requires mutable
/// references because the underlying API uses mutable pointers.
///
/// The caller must call `CFRelease` on the returned dictionary.
pub unsafe fn create_dictionary(
    keys: &mut [CFTypeRef],
    values: &mut [CFTypeRef],
) -> CFDictionaryRef {
    assert!(keys.len() == values.len());
    CFDictionaryCreate(
        std::ptr::null_mut(),
        keys.as_mut_ptr(),
        values.as_mut_ptr(),
        keys.len() as i64,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks,
    )
}
