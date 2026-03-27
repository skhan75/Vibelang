use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("embedded_stdlib.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let stdlib_dir = Path::new(&manifest_dir).join("../../stdlib/std");

    let mut entries = Vec::new();

    if stdlib_dir.is_dir() {
        let mut files: Vec<_> = fs::read_dir(&stdlib_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "yb"))
            .collect();
        files.sort();

        for file in &files {
            let name = file.file_name().unwrap().to_str().unwrap();
            let content = fs::read_to_string(file)
                .unwrap_or_default()
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            entries.push(format!("    (\"{name}\", \"{content}\")"));
            println!("cargo:rerun-if-changed={}", file.display());
        }
    }

    let code = format!(
        "pub const EMBEDDED_STDLIB: &[(&str, &str)] = &[\n{}\n];\n",
        entries.join(",\n")
    );
    fs::write(&dest, code).unwrap();

    println!("cargo:rerun-if-changed=../../stdlib/std");
}
