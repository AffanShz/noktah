//! Menanam version info dan manifest ke dalam noktah-hook.exe.
//!
//! Binary pendamping juga wajib memiliki identitas produk, versi, dan
//! manifest `asInvoker` agar tidak memicu deteksi heuristik antivirus dan
//! lolos pemeriksaan rilis.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set("ProductName", "Noktah");
    res.set(
        "FileDescription",
        "Noktah Hook - pendamping IPC untuk agent dan status kustom",
    );
    res.set("CompanyName", "Noktah");
    res.set("LegalCopyright", "Noktah");
    res.set("OriginalFilename", "noktah-hook.exe");
    res.set("InternalName", "noktah-hook");
    res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
    res.set("FileVersion", env!("CARGO_PKG_VERSION"));

    let manifest = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity type="win32" name="Noktah.Hook" version="{}.0" processorArchitecture="amd64"/>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}"/>
    </application>
  </compatibility>
</assembly>
"#,
        env!("CARGO_PKG_VERSION")
    );
    res.set_manifest(&manifest);

    if let Err(err) = res.compile() {
        println!("cargo:warning=gagal menanam resource: {err}");
    }
}
