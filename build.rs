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

use bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    if std::env::var("TARGET").unwrap().contains("-apple") {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");

        let bindings = bindgen::Builder::default()
            .header("src/keychain-wrapper.h")
            // Security Framework
            .whitelist_function("SecCopyErrorMessageString")
            .whitelist_function("^SecItem.*")
            .whitelist_var("^kSec.*")
            .whitelist_var("^errSec.*") // Error codes
            // Core foundation
            .whitelist_function("^CFString.*")
            .whitelist_var("^kCF.*")
            .whitelist_function("^CFData.*")
            .whitelist_function("^CFDictionary.*")
            .whitelist_function("CFRelease")
            .whitelist_function("CFShow")
            .whitelist_function("CFTypeRef")
            // Base types
            .whitelist_type("OSStatus")
            // Link against necessary frameworks
            .link_framework("Security")
            .link_framework("CoreFoundation")
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
