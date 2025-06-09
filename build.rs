use std::fs::{read_to_string, write};

fn main() {
    let fragments = vec![
        "pests/base.pest",
        "pests/rules.pest",
        "pests/conditions.pest",
        "pests/values.pest",
    ];

    let mut combined = String::new();

    for path in fragments {
        let contents = read_to_string(path).expect(&format!("Failed to read {}", path));
        combined.push_str(&contents);
        combined.push('\n');
    }

    write("pests/grammar.pest", combined).expect("Failed to write combined grammar");
    println!("cargo:rerun-if-changed=build.rs");
}

