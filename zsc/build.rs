const AUTOGEN_PATH: &str = "./autogen-rules/";

fn main() {
    println!("cargo::rerun-if-changed={AUTOGEN_PATH}");
    autogen_file("x86", "./src/backend/x86/instr.rs");
    autogen_file("lir", "./src/IRs/lir/instr.rs");
}

fn autogen_file(arch: &str, output_path: &str) {
    let main_py = format!("{AUTOGEN_PATH}/main.py");

    let out = std::process::Command::new("python3")
        .arg("-X")
        .arg("pycache_prefix=/dev/null")
        .arg(main_py)
        .arg("-t")
        .arg(arch)
        .output()
        .unwrap();

    std::fs::write(output_path, out.stdout.as_slice()).unwrap();
}
