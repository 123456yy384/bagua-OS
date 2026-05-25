use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target != "x86_64-unknown-none" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Assemble boot64.S with GNU as from WSL
    let asm_path = format!("{manifest_dir}/src/boot64.S");
    let obj_path = format!("{out_dir}/boot64.o");

    fn to_wsl(p: &str) -> String {
        let s = p.replace('\\', "/");
        if s.starts_with("C:") { format!("/mnt/c{}", &s[2..]) }
        else if s.starts_with("D:") { format!("/mnt/d{}", &s[2..]) }
        else { s }
    }

    let output = Command::new("wsl")
        .args(["as", "--64", "-o", &to_wsl(&obj_path), &to_wsl(&asm_path)])
        .output()
        .expect("GNU as not available. Install: wsl sudo apt install binutils");

    if !output.status.success() {
        panic!("as failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    println!("cargo:rustc-link-arg={obj_path}");
    println!("cargo:rerun-if-changed={asm_path}");
}
