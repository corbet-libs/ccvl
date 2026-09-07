#[path = "src/runtime_source.rs"]
mod runtime_source;

fn main() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rerun-if-changed=.agent/src");
    for path in runtime_source::inputs(&root).expect("runtime inputs must exist") {
        println!("cargo:rerun-if-changed={path}");
    }
    let identity = runtime_source::fingerprint(&root).expect("runtime inputs must be readable");
    println!("cargo:rustc-env=CCVL_RUNTIME_ID={identity}");
}
