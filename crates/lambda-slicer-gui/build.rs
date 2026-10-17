use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../../assets/branding/app.ico");
    println!("cargo:rerun-if-env-changed=RC");
    println!("cargo:rerun-if-env-changed=WINDRES");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon = manifest.join("../../assets/branding/app.ico");
    let icon = icon.canonicalize().expect("application icon is missing");
    // RC accepts forward slashes and quoted paths containing spaces
    let icon = icon.to_string_lossy().replace('\\', "/");
    let icon = if let Some(path) = icon.strip_prefix("//?/UNC/") {
        format!("//{path}")
    } else {
        icon.strip_prefix("//?/").unwrap_or(&icon).to_owned()
    };
    let source = out.join("app.rc");
    fs::write(&source, format!("1 ICON \"{icon}\"\n")).expect("cannot write icon resource source");

    let msvc = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    let resource = out.join(if msvc { "app.res" } else { "app.o" });
    let mut compiler = if msvc {
        let compiler = env::var_os("RC").map(PathBuf::from).or_else(|| {
            let sdk = find_msvc_tools::find_windows_sdk(env::consts::ARCH)?;
            sdk.path()
                .map(|dir| dir.join("rc.exe"))
                .find(|path| path.is_file())
        });
        let mut command = Command::new(compiler.unwrap_or_else(|| "rc.exe".into()));
        command
            .arg("/nologo")
            .arg("/c65001")
            .arg("/fo")
            .arg(&resource)
            .arg(&source);
        command
    } else {
        let mut command = Command::new(env::var_os("WINDRES").unwrap_or_else(|| "windres".into()));
        command.arg("--codepage=65001").arg("-i").arg(&source);
        command.arg("-O").arg("coff");
        command.arg("-o").arg(&resource);
        command
    };
    let status = compiler.status().expect(
        "cannot run the Windows resource compiler; install the Windows SDK or set RC (MSVC) or WINDRES (GNU)",
    );
    assert!(status.success(), "Windows icon resource compilation failed");

    // Limit the resource to the app binary, not tests or GUI examples
    println!(
        "cargo:rustc-link-arg-bin=lambda-slicer-gui={}",
        resource.display()
    );
}
