use helix_loader::grammar::fetch_grammars;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let tutor = fs::read_to_string("../runtime/tutor")?;
    let tutor = format!("const TUTOR: &str = r##\"{}\"##;", tutor);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tutor.rs");
    fs::write(&dest_path, &tutor)?;
    println!("cargo:rerun-if-changed=../runtime/tutor");

    let languages: Vec<String> = std::fs::read_to_string("languages")?
        .lines()
        .map(|s| s.to_string())
        .collect();

    fetch_grammars(Some(&languages)).expect("Failed to fetch tree-sitter grammars");

    let mut build = cc::Build::new();
    build.file("src/wasm-sysroot/wctype.c");
    build.include("src/wasm-sysroot/");
    build.compile("wctype");

    const PARSER_C: &str = "parser.c";
    const SCANNER_C: &str = "scanner.c";

    println!("cargo:rerun-if-changed=languages");
    for language in languages {
        let base_path = format!("../runtime/grammars/sources/{}/src/", language);

        let mut build = cc::Build::new();
        build
            .opt_level(3)
            .include("src/wasm-sysroot/")
            .include(&base_path);

        let parser_c = Path::new(&base_path).join(PARSER_C);
        println!("cargo:rerun-if-changed={}", parser_c.display());
        build.file(&parser_c);

        let scanner_c = Path::new(&base_path).join(SCANNER_C);
        if scanner_c.exists() {
            println!("cargo:rerun-if-changed={}", scanner_c.display());
            build.file(&scanner_c);
        }
        build.compile(&language);
    }

    Ok(())
}
