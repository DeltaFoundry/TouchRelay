use axum::extract::ws::{Message, WebSocket};
use enigo::{Enigo, Mouse, Button, Keyboard, Direction, Settings, Key};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Handle WebSocket connection
pub async fn handle_socket(mut socket: WebSocket) {
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

/// Handle incoming WebSocket message
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

            "k" => {
                // Key press: ["k", "KeyName"]
                if arr.len() < 2 {
                    return Err("Invalid key press message".to_string());
                }
                let key_name = arr[1].as_str().ok_or("Invalid key name")?;

                let key = match key_name {
                    "Escape" => Key::Escape,
                    "PageUp" => Key::PageUp,
                    "PageDown" => Key::PageDown,
                    "Delete" => Key::Backspace,  // Del button sends Backspace key
                    "Return" => Key::Return,
                    _ => return Err(format!("Unknown key: {}", key_name)),
                };

                let mut enigo = enigo.lock().await;
                enigo.key(key, Direction::Click)
                    .map_err(|e| format!("Key press failed: {}", e))?;
                info!("Key pressed: {} (mapped to {:?})", key_name, key);
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
