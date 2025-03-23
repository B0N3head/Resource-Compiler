#[cfg(windows)]
fn main() {
    // This tells Rust to build the application as a Windows GUI app (no console window)
    // Only needed on Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        if std::env::var_os("CARGO_CFG_TARGET_ENV").unwrap() == "msvc" {
            let mut res = winres::WindowsResource::new();
            
            // Check if icon file exists, otherwise skip it (don't fail the build)
            let icon_path = "assets/app_icon.ico";
            if std::path::Path::new(icon_path).exists() {
                res.set_icon(icon_path);
            } else {
                eprintln!("Warning: Icon file not found at {}", icon_path);
            }
            
            res.set("FileDescription", "Resource Compiler");
            res.set("ProductName", "Resource Compiler");
            
            if let Err(e) = res.compile() {
                eprintln!("Failed to set Windows resource: {}", e);
            }
        }
    }
}

#[cfg(not(windows))]
fn main() {
    // Nothing to do for non-Windows platforms
}
