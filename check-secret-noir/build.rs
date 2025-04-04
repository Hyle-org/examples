use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=contract");

    let output = Command::new("nargo")
        .arg("compile")
        .current_dir("contract")
        .output()
        .expect("Failed to execute nargo compile");

    if !output.status.success() {
        eprintln!(
            "nargo compile failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }

    println!(
        "nargo compile succeeded with output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
