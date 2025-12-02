// Hide console window on Windows in release mode (debug mode keeps console for logs)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    extract::ws::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
    http::{header, StatusCode},
};
use axum::extract::ws::{Message, WebSocket};
use enigo::{Enigo, Mouse, Button, Keyboard, Direction, Settings};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent},
    TrayIconBuilder, Icon,
};
use winit::event_loop::{EventLoop, ControlFlow, ActiveEventLoop};
use winit::application::ApplicationHandler;
use local_ip_address::local_ip;
use tray_icon::menu::MenuId;

// Application handler for winit event loop
struct TrayApp {
    about_id: MenuId,
    quit_id: MenuId,
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
            if event.id == self.about_id {
                info!("About menu item clicked, opening GitHub page...");
                let _ = open::that("https://github.com/DeltaFoundry/TouchRelay");
            } else if event.id == self.quit_id {
                info!("Quit menu item clicked, shutting down...");
                event_loop.exit();
            }
        }
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
    let icon = load_icon();

    // Get local IP address
    let tooltip = match local_ip() {
        Ok(ip) => {
            let url = format!("http://{}:8000/", ip);
            info!("Local access URL: {}", url);
            format!("TouchRelay - Remote Mouse & Keyboard\nConnect: {}", url)
        }
        Err(_) => {
            warn!("Failed to detect local IP address");
            "TouchRelay - Remote Mouse & Keyboard\nConnect: http://<PC_IP>:8000/".to_string()
        }
    };

    // Create tray menu
    let tray_menu = Menu::new();
    let about_item = MenuItem::new("About", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let about_id = about_item.id().clone();
    let quit_id = quit_item.id().clone();
    tray_menu.append(&about_item).unwrap();
    tray_menu.append(&quit_item).unwrap();

    // Build tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
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
        about_id,
        quit_id,
    };

    // Run event loop in main thread
    event_loop.set_control_flow(ControlFlow::Wait);
    let _ = event_loop.run_app(&mut app);

    info!("TouchRelay stopped");
}

async fn run_server() {
    // Build router with embedded static files
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/static/style.css", get(css_handler))
        .route("/static/app.js", get(js_handler))
        .route("/static/icon.ico", get(icon_handler));

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

fn load_icon() -> Icon {
    // Load embedded icon from binary
    let icon_bytes = include_bytes!("../static/icon.ico");

    match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            match Icon::from_rgba(rgba.into_raw(), width, height) {
                Ok(icon) => {
                    info!("Loaded embedded icon ({}x{})", width, height);
                    return icon;
                }
                Err(e) => {
                    warn!("Failed to create icon from embedded image: {}", e);
                }
            }
        }
        Err(e) => {
            warn!("Failed to load embedded icon: {}", e);
        }
    }

    // Fallback: create a simple default icon
    info!("Using default icon");
    create_default_icon()
}

fn create_default_icon() -> Icon {
    // Create a simple 32x32 icon with a solid color
    let size = 32;
    let mut rgba_data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            // Create a simple gradient icon
            let r = ((x as f32 / size as f32) * 255.0) as u8;
            let g = ((y as f32 / size as f32) * 255.0) as u8;
            let b = 180;
            let a = 255;

            rgba_data.extend_from_slice(&[r, g, b, a]);
        }
    }

    Icon::from_rgba(rgba_data, size, size).expect("Failed to create default icon")
}

// Embed static files at compile time
async fn index_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../static/index.html")
    )
}

async fn css_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/style.css")
    )
}

async fn js_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../static/app.js")
    )
}

async fn icon_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/x-icon")],
        include_bytes!("../static/icon.ico").as_slice()
    )
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    info!("WebSocket connection established");

    // Create Enigo instance for this connection
    let enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => Arc::new(Mutex::new(e)),
        Err(err) => {
            error!("Failed to create Enigo instance: {}", err);
            return;
        }
    };

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_message(&text, Arc::clone(&enigo)).await {
                    warn!("Failed to handle message: {} - Error: {}", text, e);
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

async fn handle_message(text: &str, enigo: Arc<Mutex<Enigo>>) -> Result<(), String> {
    let msg: Value = serde_json::from_str(text)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    if let Value::Array(arr) = msg {
        if arr.is_empty() {
            return Err("Empty message array".to_string());
        }

        let cmd = arr[0].as_str().ok_or("Invalid command type")?;

        match cmd {
            "m" => {
                // Mouse move: ["m", dx, dy]
                if arr.len() < 3 {
                    return Err("Invalid mouse move message".to_string());
                }
                let dx = arr[1].as_i64().ok_or("Invalid dx")? as i32;
                let dy = arr[2].as_i64().ok_or("Invalid dy")? as i32;

                let mut enigo = enigo.lock().await;
                enigo.move_mouse(dx, dy, enigo::Coordinate::Rel)
                    .map_err(|e| format!("Mouse move failed: {}", e))?;
            }

            "b" => {
                // Button click: ["b", "l"|"r", 1|2]
                if arr.len() < 3 {
                    return Err("Invalid button click message".to_string());
                }
                let button_type = arr[1].as_str().ok_or("Invalid button type")?;
                let click_count = arr[2].as_u64().ok_or("Invalid click count")? as u32;

                let button = match button_type {
                    "l" => Button::Left,
                    "r" => Button::Right,
                    _ => return Err(format!("Unknown button type: {}", button_type)),
                };

                let mut enigo = enigo.lock().await;
                for _ in 0..click_count {
                    enigo.button(button, Direction::Click)
                        .map_err(|e| format!("Button click failed: {}", e))?;
                    // Add small delay between double clicks
                    if click_count > 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                }
            }

            "w" => {
                // Mouse wheel: ["w", dy]
                if arr.len() < 2 {
                    return Err("Invalid wheel message".to_string());
                }
                let dy = arr[1].as_i64().ok_or("Invalid dy")? as i32;

                let mut enigo = enigo.lock().await;
                // Convert dy to scroll amount (positive = scroll up, negative = scroll down)
                enigo.scroll(dy, enigo::Axis::Vertical)
                    .map_err(|e| format!("Wheel scroll failed: {}", e))?;
            }

            "t" => {
                // Text input: ["t", "text content"]
                if arr.len() < 2 {
                    return Err("Invalid text message".to_string());
                }
                let text_content = arr[1].as_str().ok_or("Invalid text content")?;

                let mut enigo = enigo.lock().await;
                enigo.text(text_content)
                    .map_err(|e| format!("Text input failed: {}", e))?;
            }

            "ping" => {
                // Heartbeat - do nothing
                info!("Ping received");
            }

            _ => {
                return Err(format!("Unknown command: {}", cmd));
            }
        }

        Ok(())
    } else {
        Err("Message is not an array".to_string())
    }
}
