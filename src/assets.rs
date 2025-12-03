use axum::{response::IntoResponse, http::{header, StatusCode}};
use tray_icon::Icon;
use tracing::{info, warn};

/// Load tray icon from embedded resources
pub fn load_icon() -> Icon {
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

/// Create a default gradient icon
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

// Static file handlers (embedded at compile time)

/// Serve index.html
pub async fn index_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../static/index.html")
    )
}

/// Serve style.css
pub async fn css_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/style.css")
    )
}

/// Serve app.js
pub async fn js_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../static/app.js")
    )
}

/// Serve icon.ico
pub async fn icon_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/x-icon")],
        include_bytes!("../static/icon.ico").as_slice()
    )
}
