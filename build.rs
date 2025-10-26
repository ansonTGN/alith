fn main() {
    // Link against nettle library on Windows
    if cfg!(target_os = "windows") {
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
