use std::{env, str};

fn main() {
    if rustc::is_feature_flaggable().unwrap_or(false) {
        println!("cargo:rustc-cfg=value_bag_capture_const_type_id");
    } else if target_arch_is_any(&["x86_64", "aarch64"]) && target_os_is_any(&["windows", "linux", "macos"]) {
        println!("cargo:rustc-cfg=value_bag_capture_ctor");
    } else {
        println!("cargo:rustc-cfg=value_bag_capture_fallback");
    }
}

fn target_arch_is_any(archs: &[&str]) -> bool {
    cargo_env_is_any("CARGO_CFG_TARGET_ARCH", archs)
}

fn target_os_is_any(families: &[&str]) -> bool {
    cargo_env_is_any("CARGO_CFG_TARGET_OS", families)
}

fn cargo_env_is_any(env: &str, values: &[&str]) -> bool {
    match env::var(env) {
        Ok(var) if values.contains(&&*var) => true,
        _ => false,
    }
}
