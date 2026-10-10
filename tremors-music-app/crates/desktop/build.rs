// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=resources/app.rc");
        println!("cargo:rerun-if-changed=../../../assets/TremorsMusic.ico");
        let version = std::env::var("CARGO_PKG_VERSION").expect("Missing Cargo package version");
        let numeric_version = format!("{},0", version.replace('.', ","));
        let icon = std::path::Path::new("../../../assets/TremorsMusic.ico")
            .canonicalize()
            .expect("Could not locate the application icon")
            .to_string_lossy()
            .replace('\\', "/")
            .trim_start_matches("//?/")
            .to_owned();
        let resource = std::fs::read_to_string("resources/app.rc")
            .expect("Could not read Windows application resources")
            .replace("@VERSION@", &version)
            .replace("@NUMERIC_VERSION@", &numeric_version)
            .replace("@ICON@", &icon);
        let output =
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Missing OUT_DIR"))
                .join("app.rc");
        std::fs::write(&output, resource)
            .expect("Could not generate Windows application resources");
        embed_resource::compile(output, embed_resource::NONE)
            .manifest_optional()
            .expect("Could not compile Windows application resources");
    }
}
