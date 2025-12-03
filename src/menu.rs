use tray_icon::menu::{Menu, MenuItem, MenuId};
use tracing::{info, error};
use local_ip_address::local_ip;

// Menu action enum for handling menu events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    OpenWeb,
    ToggleStartup,
    About,
    Quit,
    None,
}

// Tray menu structure with all menu items
pub struct TrayMenu {
    menu: Menu,
    open_web_id: MenuId,
    startup_id: MenuId,
    about_id: MenuId,
    quit_id: MenuId,
}

impl TrayMenu {
    /// Create a new tray menu with current state
    pub fn new() -> Self {
        let menu = Menu::new();

        // Create menu items
        let open_web_item = MenuItem::new("Open Web Interface", true, None);

        let is_startup_enabled = crate::startup::is_startup_enabled();
        let startup_text = if is_startup_enabled {
            "✓ Start with Windows"
        } else {
            "Start with Windows"
        };
        let startup_item = MenuItem::new(startup_text, true, None);

        let about_item = MenuItem::new("About", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        // Get menu IDs
        let open_web_id = open_web_item.id().clone();
        let startup_id = startup_item.id().clone();
        let about_id = about_item.id().clone();
        let quit_id = quit_item.id().clone();

        // Append items to menu
        menu.append(&open_web_item).unwrap();
        menu.append(&startup_item).unwrap();
        menu.append(&about_item).unwrap();
        menu.append(&quit_item).unwrap();

        info!("Tray menu created");

        Self {
            menu,
            open_web_id,
            startup_id,
            about_id,
            quit_id,
        }
    }

    /// Handle menu event and return the corresponding action
    pub fn handle_event(&self, event_id: &MenuId) -> MenuAction {
        if event_id == &self.open_web_id {
            MenuAction::OpenWeb
        } else if event_id == &self.startup_id {
            MenuAction::ToggleStartup
        } else if event_id == &self.about_id {
            MenuAction::About
        } else if event_id == &self.quit_id {
            MenuAction::Quit
        } else {
            MenuAction::None
        }
    }

    /// Get the menu
    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    /// Execute menu action and return whether menu should be updated
    pub fn execute_action(&self, action: MenuAction) -> bool {
        match action {
            MenuAction::OpenWeb => {
                info!("Opening web interface...");
                open_web_interface();
                false
            }
            MenuAction::ToggleStartup => {
                info!("Toggling startup...");
                crate::startup::toggle_startup();
                true // Return true to indicate menu should be updated
            }
            MenuAction::About => {
                info!("Opening GitHub page...");
                if let Err(e) = open::that("https://github.com/DeltaFoundry/TouchRelay") {
                    error!("Failed to open GitHub page: {}", e);
                }
                false
            }
            MenuAction::Quit => {
                info!("Quit action triggered");
                false // Quit is handled by caller
            }
            MenuAction::None => {
                false
            }
        }
    }
}

/// Open the web interface in the default browser
fn open_web_interface() {
    match local_ip() {
        Ok(ip) => {
            let url = format!("http://{}:8000/", ip);
            info!("Opening web interface: {}", url);
            if let Err(e) = open::that(&url) {
                error!("Failed to open web interface: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to get local IP address: {}", e);
            // Fallback to localhost
            let url = "http://127.0.0.1:8000/";
            info!("Opening web interface (localhost): {}", url);
            if let Err(e) = open::that(url) {
                error!("Failed to open web interface: {}", e);
            }
        }
    }
}
