use std::path::Path;

fn main() {
    if std::env::consts::OS == "macos" {
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let mupdf_path = Path::new(&manifest_dir)
            .join("../../thirdparty/mupdf/build/release")
            .canonicalize()
            .unwrap();
        println!("cargo:rustc-link-search={}", mupdf_path.display());
    }
}
