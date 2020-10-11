use std::{env, process::Command, str};

fn main() {
    if rustc_is_nightly().unwrap_or(false) {
        println!("cargo:rustc-cfg=value_bag_const_type_id");
    }
}

fn rustc_is_nightly() -> Option<bool> {
    let rustc = match env::var_os("RUSTC") {
        Some(rustc) => rustc,
        None => return None,
    };

    let output = match Command::new(rustc).arg("--version").output() {
        Ok(output) => output,
        Err(_) => return None,
    };

    let version = match str::from_utf8(&output.stdout) {
        Ok(version) => version,
        Err(_) => return None,
    };

    Some(version.contains("-nightly"))
}
