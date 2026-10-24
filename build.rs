//! Windows only: embed the app icon and version info into `docplane.exe`.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/docplane.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/docplane.ico")
        .set("ProductName", "DocPlane.it")
        .set(
            "FileDescription",
            "DocPlane.it: videoscrittura, foglio di calcolo e presentazione",
        )
        .set("CompanyName", "Lorenzo D'Apolito")
        .set(
            "LegalCopyright",
            "Copyright (c) Lorenzo D'Apolito. MIT OR Apache-2.0.",
        )
        .set("OriginalFilename", "DocPlane.it.exe")
        .set("InternalName", "docplane");
    if let Err(e) = res.compile() {
        println!("cargo:warning=docplane.exe built without icon/version resources: {e}");
    }
}
