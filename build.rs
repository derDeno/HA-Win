fn main() {
    println!("cargo:rerun-if-changed=icon.ico");
    println!("cargo:rerun-if-changed=app.manifest");
    winresource::WindowsResource::new()
        .set_icon("icon.ico")
        .set_manifest_file("app.manifest")
        .set("ProductName", "HA Win")
        .set("FileDescription", "Home Assistant Windows client")
        .compile()
        .expect("Could not compile Windows resources (install the Windows SDK)");
}
