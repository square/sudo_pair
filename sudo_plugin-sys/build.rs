// Copyright 2018 Square Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

use std::env;
use std::path::PathBuf;

fn main() {
    let bindings_path = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join("bindings.rs");

    bindings::generate(&bindings_path);
}

#[cfg(not(feature = "bindgen"))]
mod bindings {
    use std::fs;
    use std::path::Path;

    #[cfg(target_arch = "aarch64")]
    const TARGET_ARCH : &str = "aarch64";

    #[cfg(target_arch = "x86_64")]
    const TARGET_ARCH : &str = "x86-64";

    #[cfg(target_arch = "x86")]
    const TARGET_ARCH : &str = "x86";

    const SUDO_PLUGIN_API_VERSIONS : &[&str] = &[
        #[cfg(feature = "min_sudo_plugin_1_9")]
        "1.9",

        #[cfg(feature = "min_sudo_plugin_1_12")]
        "1.12",

        #[cfg(feature = "min_sudo_plugin_1_14")]
        "1.14",
    ];

    pub fn generate(out_path: &Path) {
        let in_path = format!(
            "src/bindings/sudo_plugin-{}.{}.rs",
            SUDO_PLUGIN_API_VERSIONS.last().unwrap(),
            TARGET_ARCH,
        );

        fs::copy(in_path, out_path).unwrap();
    }
}

#[cfg(feature = "bindgen")]
mod bindings {
    use bindgen::builder;
    use std::path::Path;

    pub fn generate(out_path: &Path) {
      builder()
        .clang_arg("-I/usr/include")
        .clang_arg("-I/usr/local/include")
        .header("include/bindings.h")
        .generate()
        .unwrap()
        .write_to_file(out_path)
        .unwrap()
    }
}
