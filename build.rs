use std::{env, process::Command, str};

fn main() {
    if rustc::is_feature_flaggable().unwrap_or(false) {
        println!("cargo:rustc-cfg=value_bag_capture_const_type_id");
    } else if target_arch_is_any(&["x86_64", "aarch64"]) && target_os_is_any(&["windows", "linux", "macos"]) {
        println!("cargo:rustc-cfg=value_bag_capture_ctor");
    } else {
        println!("cargo:rustc-cfg=value_bag_capture_fallback");
    }
}

fn target_arch_is_any(targets: &[&str]) -> bool {
    match env::var("CARGO_CFG_TARGET_ARCH") {
        Ok(arch) if targets.contains(&&*arch) => true,
        _ => false,
    }
}

fn target_os_is_any(family: &[&str]) -> bool {
    match env::var("CARGO_CFG_TARGET_OS") {
        Ok(arch) if targets.contains(&&*arch) => true,
        _ => false,
    }
}
