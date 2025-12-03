// Hide console window on Windows in release mode (debug mode keeps console for logs)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod menu;
mod startup;
mod handler;
mod assets;

use axum::{
    extract::ws::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
};
use tracing::{info, warn};
use tray_icon::{
    menu::MenuEvent,
    TrayIconBuilder, TrayIcon,
};
use winit::event_loop::{EventLoop, ControlFlow, ActiveEventLoop};
use winit::application::ApplicationHandler;
use local_ip_address::local_ip;

use menu::{TrayMenu, MenuAction};

// Application handler for winit event loop
struct TrayApp {
    tray_icon: TrayIcon,
    tray_menu: TrayMenu,
}

impl ApplicationHandler for TrayApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Called when the application is resumed
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
        // We don't have any windows, so this is not used
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Check for menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let action = self.tray_menu.handle_event(&event.id);

            // Execute action and check if we need to quit or update menu
            let should_update_menu = self.tray_menu.execute_action(action);

            if action == MenuAction::Quit {
                event_loop.exit();
            } else if should_update_menu {
                self.update_menu();
            }
        }
    }
}

impl TrayApp {
    /// Update the tray menu to reflect current startup state
    fn update_menu(&mut self) {
        let new_menu = TrayMenu::new();
        self.tray_icon.set_menu(Some(Box::new(new_menu.menu().clone())));
        self.tray_menu = new_menu;
        info!("Menu updated with current startup state");
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting TouchRelay server...");

    // Create winit event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Load icon
    let icon = assets::load_icon();

    // Get local IP address
    let tooltip = match local_ip() {
        Ok(ip) => {
            let url = format!("http://{}:8000/", ip);
            info!("Local access URL: {}", url);
            format!("TouchRelay\n{}", url)
        }
        Err(_) => {
            warn!("Failed to detect local IP address");
            "TouchRelay\nhttp://<PC_IP>:8000/".to_string()
        }
    };

    // Create tray menu
    let tray_menu = TrayMenu::new();

    // Build tray icon
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu.menu().clone()))
        .with_tooltip(&tooltip)
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    info!("System tray icon created");

    // Start web server in a separate thread
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            run_server().await;
        });
    });

    // Create application handler
    let mut app = TrayApp {
        tray_icon,
        tray_menu,
    };

    // Run event loop in main thread
    event_loop.set_control_flow(ControlFlow::Wait);
    let _ = event_loop.run_app(&mut app);

    info!("TouchRelay stopped");
}

async fn run_server() {
    // Build router with embedded static files
    let app = Router::new()
        .route("/", get(assets::index_handler))
        .route("/ws", get(ws_handler))
        .route("/static/style.css", get(assets::css_handler))
        .route("/static/app.js", get(assets::js_handler))
        .route("/static/icon.ico", get(assets::icon_handler));

    let addr = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("Server listening on http://{}", addr);
    info!("Access from mobile: http://<PC_IP>:8000/");

    // Run server
    axum::serve(listener, app)
        .await
        .unwrap();

    info!("TouchRelay server stopped");
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handler::handle_socket)
}
