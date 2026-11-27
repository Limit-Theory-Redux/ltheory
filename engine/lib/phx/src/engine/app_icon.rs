//! The application's icon where the system takes it from the running app rather than a bundle.
//!
//! On macOS a binary run without an app bundle (`ltr` straight from `target/`) gets the generic
//! executable icon in the Dock: the icon of a bundle comes from its `Info.plist`, and a window's
//! icon is ignored. Setting the application's icon image at startup gives the Dock the game's icon
//! however it was started. Elsewhere the icon is the executable's own (Windows embeds it at build).

/// The icon file, relative to the directory the game runs from (the repository's root, or a
/// bundle's resources, where it is set already).
#[cfg(target_os = "macos")]
const ICON_PATH: &str = "res/images/LTR-Icon.icns";

/// Gives the running application the game's icon, if the icon file is there. Called on the main
/// thread once the event loop runs; does nothing off it.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)] // AppKit's image and application calls are unsafe in objc2.
pub(crate) fn set_app_icon() {
    use objc2::ClassType;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::{MainThreadMarker, NSString};

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    if !std::path::Path::new(ICON_PATH).exists() {
        return;
    }
    let path = NSString::from_str(ICON_PATH);
    // SAFETY: on the main thread (`mtm`), with a valid path string; a file that isn't an image
    // gives `None`, which leaves the icon as it was.
    if let Some(image) = unsafe { NSImage::initWithContentsOfFile(NSImage::alloc(), &path) } {
        let app = NSApplication::sharedApplication(mtm);
        // SAFETY: on the main thread, with an image AppKit made.
        unsafe { app.setApplicationIconImage(Some(&image)) };
    }
}

/// Elsewhere the executable carries its icon.
#[cfg(not(target_os = "macos"))]
pub(crate) fn set_app_icon() {}
