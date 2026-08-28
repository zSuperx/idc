const AUTOGEN_PATH: &str = "./spec";

fn main() {
    println!("cargo::rerun-if-changed={AUTOGEN_PATH}");
    autogen_file("x86", "./src/backends/x86/asm/instr.rs");
    autogen_file("x86MIR", "./src/backends/x86/MIR/instr.rs");
    autogen_file("stir", "./src/isa/instr.rs");
}

fn autogen_file(arch: &str, output_path: &str) {
    let script_file = format!("{AUTOGEN_PATH}/{arch}.pl");
    let out = std::process::Command::new(&script_file).output().unwrap();

    if !out.status.success() {
        eprintln!("ERROR WHILE GENERATING ISA FILES");
        eprintln!(
            "{script_file} {}\n\n{}",
            out.status,
            str::from_utf8(&out.stderr).unwrap()
        );
        std::process::exit(1);
    }

    std::fs::write(output_path, out.stdout.as_slice()).unwrap();
}
