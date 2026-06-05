fn main() {
    autogen_file("x86");
    autogen_file("lir");
    autogen_file("riscv");
}

fn autogen_file(arch: &str) {
    let autogen_path = "./src/autogen/";
    let python_path = format!("{autogen_path}/main.py");
    let output_path = format!("{autogen_path}/{arch}/mod.rs");

    let out = std::process::Command::new("python3")
        .arg(python_path)
        .arg("-t")
        .arg(arch)
        .output()
        .unwrap();

    std::fs::write(output_path, out.stdout.as_slice()).unwrap();
}
