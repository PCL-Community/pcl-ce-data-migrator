use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

fn main() {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let material_dir = Path::new(&manifest_dir).join("material-1.0");

    if !material_dir.join("material.slint").exists() {
        let url = "https://material.slint.dev/zip/material-1.0.zip";
        let mut bytes = Vec::new();
        ureq::get(url)
            .call()
            .unwrap()
            .into_reader()
            .read_to_end(&mut bytes)
            .unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        archive.extract(&manifest_dir).unwrap();
        println!("cargo:warning=Downloaded and extracted material-1.0");
    }

    let config = slint_build::CompilerConfiguration::new().with_library_paths(HashMap::from([(
        "material".to_string(),
        material_dir.join("material.slint"),
    )]));
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
