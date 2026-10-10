// Copyright 2022-2025 Zed Industries, Inc.
// Copyright 2026 Tremors and contributors.
// SPDX-License-Identifier: Apache-2.0
// Local modification: embed checked-in shader bytecode for portable release builds.
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") || cfg!(debug_assertions) {
        return;
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("build output directory"));
    let modules = [
        "quad",
        "shadow",
        "path_rasterization",
        "path_sprite",
        "underline",
        "monochrome_sprite",
        "subpixel_sprite",
        "polychrome_sprite",
        "emoji_rasterization",
    ];
    let mut bindings = String::new();
    for module in modules {
        for (suffix, target) in [("vs", "VERTEX"), ("ps", "FRAGMENT")] {
            let path = root.join("shaders").join(format!("{module}_{suffix}.cso"));
            println!("cargo:rerun-if-changed={}", path.display());
            let bytes = fs::read(&path)
                .unwrap_or_else(|e| panic!("Missing release shader {}: {e}", path.display()));
            assert!(
                bytes.starts_with(b"DXBC"),
                "Invalid DXBC shader {}",
                path.display()
            );
            bindings.push_str(&format!(
                "const {}_{}_BYTES: &[u8] = include_bytes!({:?});\n",
                module.to_uppercase(),
                target,
                path
            ));
        }
    }
    fs::write(out.join("shaders_bytes.rs"), bindings).expect("write embedded shader bindings");
}
