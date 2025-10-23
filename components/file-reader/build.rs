fn main() {
    println!("cargo:rerun-if-changed=wit");
    println!("cargo:rerun-if-changed=build.rs");
}
