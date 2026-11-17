//! Menanam version info dan manifest ke dalam exe.
//!
//! Binary tanpa identitas (tanpa nama produk, versi, dan manifest) adalah
//! pemicu utama deteksi heuristik antivirus: tidak ada yang bisa diperiksa,
//! jadi mesin memutuskan berdasarkan kemiripan pola. Ini juga alasan
//! teknis — bukan kosmetik.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "Noktah");
    res.set("FileDescription", "Noktah - notch interaktif untuk Windows");
    res.set("CompanyName", "Noktah");
    res.set("LegalCopyright", "Noktah");
    res.set("OriginalFilename", "noktah-native.exe");
    res.set("InternalName", "noktah-native");
    res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
    res.set("FileVersion", env!("CARGO_PKG_VERSION"));

    // asInvoker: aplikasi ini tidak butuh hak admin. Manifest yang jujur
    // membuat perilakunya bisa diprediksi, termasuk oleh pemindai.
    let manifest = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity type="win32" name="Noktah.App" version="{}.0" processorArchitecture="amd64"/>
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
        // Jangan gagalkan build: resource hanya menambah kredibilitas.
        println!("cargo:warning=gagal menanam resource: {err}");
    }
}
