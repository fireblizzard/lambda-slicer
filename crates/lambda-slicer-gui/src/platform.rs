//! What the desktop does for the app, such as showing a file in its folder

use std::path::Path;
use std::process::Command;

/// Show `file` selected in Explorer on Windows, or open its folder with the
/// desktop's opener on Linux. The file manager runs on its own, so nothing
/// waits for it to close. Fails when the opener cannot be started, or the
/// path names no folder
pub fn show_in_folder(file: &Path) -> std::io::Result<()> {
    let mut command = opener(file).ok_or(std::io::ErrorKind::NotFound)?;
    let mut child = command.spawn()?;

    // Reaped on a thread of its own, so a finished opener leaves nothing
    // behind and the window never waits
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(windows)]
fn opener(file: &Path) -> Option<Command> {
    use std::os::windows::process::CommandExt;

    // Explorer reads `/select,` and the quoted path as one argument of its
    // own form, so it is passed as written rather than quoted again
    let mut select = std::ffi::OsString::from("/select,\"");
    select.push(file.as_os_str());
    select.push("\"");

    let mut command = Command::new("explorer.exe");
    command.raw_arg(select);
    Some(command)
}

#[cfg(target_os = "macos")]
fn opener(file: &Path) -> Option<Command> {
    let mut command = Command::new("open");
    command.arg("-R").arg(file);
    Some(command)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn opener(file: &Path) -> Option<Command> {
    let folder = file.parent()?;
    let mut command = Command::new("xdg-open");
    command.arg(folder);
    Some(command)
}
