// "Hey, before you build Rust, please compile this C++ file."

fn main() {
    // Compile the C++ code
    cc::Build::new()
        .file("src/inspector.cpp")
        .cpp(true) // Switch to C++ compiler (g++)
        .compile("inspector"); // Output library name

    // Tell Cargo to re-run this script if the C++ file changes
    println!("cargo:rerun-if-changed=src/inspector.cpp");
}