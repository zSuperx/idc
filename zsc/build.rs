const AUTOGEN_PATH: &'static str = "./src/arch/autogen/";

fn main() {
    println!("cargo::rerun-if-changed={AUTOGEN_PATH}");
    autogen_file("x86");
    autogen_file("lir");
    autogen_file("riscv");
}

fn autogen_file(arch: &str) {
    let main_py = format!("{AUTOGEN_PATH}/main.py");
    let autogen_output_path = format!("./src/arch/{arch}/instr.rs");

    let out = std::process::Command::new("python3")
        .arg("-X")
        .arg("pycache_prefix=/dev/null")
        .arg(main_py)
        .arg("-t")
        .arg(arch)
        .output()
        .unwrap();

    std::fs::write(autogen_output_path, out.stdout.as_slice()).unwrap();
}
