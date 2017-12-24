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

#![deny(warnings)]

#[cfg(target_os = "macos")]
mod keychain;

fn main() {
    let account = keychain::Account {
        name: "foo".to_string(),
        password: "very safe password".to_string(),
    };
    let service = "fancy-service";
    println!(
        "Delete: {:?}",
        keychain::delete_generic_passwords_by_service(service)
    );
    println!(
        "Add: {:?}",
        keychain::add_generic_password(service, &account)
    );
    println!(
        "Get: {:?}",
        keychain::find_generic_password_by_service(service)
    );
    println!(
        "Cleanup: {:?}",
        keychain::delete_generic_passwords_by_service(service)
    );
}
