fn main() {
    // Pass TARGET to main code so we can find the sidecar binary
    println!(
        "cargo:rustc-env=TARGET_TRIPLE={}",
        std::env::var("TARGET").unwrap()
    );
    tauri_build::build()
}
