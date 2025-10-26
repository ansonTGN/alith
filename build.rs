fn main() {
    // Link against nettle library on Windows
    if cfg!(target_os = "windows") {
        // Try pkg-config first (preferred method)
        if let Ok(output) = std::process::Command::new("pkg-config")
            .args(&["--libs", "--cflags", "nettle"])
            .env("PKG_CONFIG_ALLOW_SYSTEM_LIBS", "1")
            .env("PKG_CONFIG_ALLOW_SYSTEM_CFLAGS", "1")
            .output()
        {
            if output.status.success() {
                let flags = String::from_utf8_lossy(&output.stdout);
                for flag in flags.split_whitespace() {
                    if flag.starts_with("-L") {
                        println!("cargo:rustc-link-search=native={}", &flag[2..]);
                    } else if flag.starts_with("-l") {
                        println!("cargo:rustc-link-lib={}", &flag[2..]);
                    }
                }
                return;
            }
        }
        
        // Fallback to environment variables
        if let Ok(lib_dir) = std::env::var("NETTLE_LIB_DIR") {
            println!("cargo:rustc-link-search=native={}", lib_dir);
            println!("cargo:rustc-link-lib=nettle");
        }
        
        // Also try vcpkg if available
        if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
            let vcpkg_lib = format!("{}\\installed\\x64-windows\\lib", vcpkg_root);
            println!("cargo:rustc-link-search=native={}", vcpkg_lib);
            println!("cargo:rustc-link-lib=nettle");
        }
    }
}
