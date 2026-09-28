// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const SWIFT_LIB: &str = "swvirtualdisplay";

fn run_swift(args: &[&str], dir: &Path) -> std::process::Output {
    let out = Command::new("swift")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run `swift` - is the toolchain installed?");
    if !out.status.success() {
        panic!(
            "swift {:?} failed:\n{}\n{}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    out
}

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let scratch = out_dir.join("swift-build");
    let scratch_s = scratch.to_str().unwrap();
    let base = ["build", "-c", "release", "--scratch-path", scratch_s];

    run_swift(&base, &manifest);

    let mut show = base.to_vec();
    show.push("--show-bin-path");
    let bin = run_swift(&show, &manifest);
    let bin_dir = PathBuf::from(String::from_utf8(bin.stdout).unwrap().trim());

    let archive = format!("lib{SWIFT_LIB}.a");
    let link_dir = out_dir.join("swift-static");
    fs::create_dir_all(&link_dir).unwrap();
    fs::copy(bin_dir.join(&archive), link_dir.join(&archive)).unwrap_or_else(|e| {
        panic!(
            "{archive} not found in {} ({e}) - declare the product as `type: .static` in Package.swift",
            bin_dir.display()
        )
    });

    println!("cargo:rustc-link-search=native={}", link_dir.display());
    println!("cargo:rustc-link-lib=static={SWIFT_LIB}");

    println!("cargo:rustc-link-search=native=/usr/lib/swift");
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");

    let swiftc = Command::new("xcrun")
        .args(["--find", "swiftc"])
        .output()
        .unwrap();
    let swiftc = String::from_utf8(swiftc.stdout).unwrap();
    let toolchain_usr = PathBuf::from(swiftc.trim())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    println!(
        "cargo:rustc-link-search=native={}",
        toolchain_usr.join("lib/swift/macosx").display()
    );

    for fw in ["Foundation", "CoreGraphics", "AppKit"] {
        println!("cargo:rustc-link-lib=framework={fw}");
    }

    println!("cargo:rerun-if-changed=does-not-exist");
    println!("cargo:rerun-if-changed=Package.swift");
    println!("cargo:rerun-if-changed=Sources");
}
