fn main() {
    println!("cargo:rerun-if-changed=pages/");
    println!("cargo:rerun-if-changed=api/");
    println!("cargo:rerun-if-changed=components/");
    println!("cargo:rerun-if-changed=sections/");
    println!("cargo:rerun-if-changed=app.rs");
    println!("cargo:rerun-if-changed=middleware.rs");

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let out = std::path::Path::new(&out_dir);
    vega_router::generate_all(".", out).expect("vega codegen");
}
