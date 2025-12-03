use winreg::enums::*;
use winreg::RegKey;
use tracing::{info, error};

const APP_NAME: &str = "TouchRelay";

/// Check if the application is set to start with Windows
pub fn is_startup_enabled() -> bool {
    match get_startup_registry_key(false) {
        Ok(key) => {
            match key.get_value::<String, _>(APP_NAME) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Enable startup with Windows
fn enable_startup() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let key = get_startup_registry_key(true)?;
    key.set_value(APP_NAME, &exe_path_str)?;

    info!("Startup enabled: {}", exe_path_str);
    Ok(())
}

/// Disable startup with Windows
fn disable_startup() -> Result<(), Box<dyn std::error::Error>> {
    let key = get_startup_registry_key(true)?;
    key.delete_value(APP_NAME)?;

    info!("Startup disabled");
    Ok(())
}

/// Toggle startup with Windows
pub fn toggle_startup() {
    if is_startup_enabled() {
        match disable_startup() {
            Ok(_) => info!("Successfully disabled startup"),
            Err(e) => error!("Failed to disable startup: {}", e),
        }
    } else {
        match enable_startup() {
            Ok(_) => info!("Successfully enabled startup"),
            Err(e) => error!("Failed to enable startup: {}", e),
        }
    }
}

/// Get the Windows registry key for startup programs
fn get_startup_registry_key(writable: bool) -> Result<RegKey, std::io::Error> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if writable {
        hkcu.open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_WRITE,
        )
    } else {
        hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
    }
}
