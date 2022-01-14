
fn main() {
    let mut cmake = cmake::Config::new(".");
    let out_dir = cmake.build();
    let lib = out_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib.display());
}
