use std::fs::{read_to_string, write};

fn main() {
    let fragments = vec![
        "pests/base.pest",
        "pests/rules.pest",
        "pests/conditions.pest",
        "pests/values.pest",
    ];

    let mut combined = String::new();

    for path in &fragments {
        let contents = read_to_string(path).expect(&format!("Failed to read {}", path));
        combined.push_str(&contents);
        combined.push('\n');

        // Tell Cargo to rerun the build script if this fragment file changes
        println!("cargo:rerun-if-changed={}", path);
    }

    write("pests/grammar.pest", combined).expect("Failed to write combined grammar");

    // Also rerun if build.rs itself changes
    println!("cargo:rerun-if-changed=build.rs");

    // And rerun if the output file is deleted (optional but good practice)
    println!("cargo:rerun-if-changed=pests/grammar.pest");
}
