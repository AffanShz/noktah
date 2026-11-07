//! Klien obrolan AI dengan streaming SSE.
//!
//! Semua penyedia memakai bentuk API yang sama (OpenAI-compatible), jadi satu
//! implementasi cukup untuk enam penyedia — hanya alamat dasar dan model yang
//! berbeda.

use noktah_core::model::Role;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub struct AiRequest {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub messages: Vec<(Role, String)>,
}

pub enum AiEvent {
    Delta(String),
    Done,
    Error(String),
}

pub struct AiHandle {
    pub rx: Receiver<AiEvent>,
    pub tx: Sender<AiRequest>,
}

pub fn spawn(ctx: egui::Context) -> AiHandle {
    let (tx_event, rx_event) = channel();
    let (tx_req, rx_req) = channel::<AiRequest>();

    let _ = std::thread::Builder::new()
        .name("noktah-ai".into())
        .stack_size(256 * 1024)
        .spawn(move || {
            while let Ok(req) = rx_req.recv() {
                match stream(&req, &tx_event) {
                    Ok(()) => {
                        let _ = tx_event.send(AiEvent::Done);
                    }
                    Err(err) => {
                        let _ = tx_event.send(AiEvent::Error(err));
                    }
                }
                ctx.request_repaint();
            }
        });

    AiHandle {
        rx: rx_event,
        tx: tx_req,
    }
}

fn stream(req: &AiRequest, tx: &Sender<AiEvent>) -> Result<(), String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .build();

    let pesan: Vec<serde_json::Value> = req
        .messages
        .iter()
        .map(|(role, content)| {
            let role = match role {
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            serde_json::json!({ "role": role, "content": content })
        })
        .collect();

    let body = serde_json::json!({
        "model": req.model,
        "messages": pesan,
        "stream": true,
    });

    let url = format!("{}/chat/completions", req.base_url.trim_end_matches('/'));

    let mut request = agent.post(&url).set("Content-Type", "application/json");
    if !req.api_key.trim().is_empty() {
        request = request.set("Authorization", &format!("Bearer {}", req.api_key.trim()));
    }

    let response = request.send_string(&body.to_string()).map_err(|e| match e {
        ureq::Error::Status(code, _) => format!("HTTP {code}"),
        other => other.to_string(),
    })?;

    let reader = BufReader::new(response.into_reader());

    for line in reader.lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() || !line.starts_with("data:") {
            continue;
        }
        let payload = line[5..].trim();
        if payload == "[DONE]" {
            break;
        }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
            if let Some(text) = json["choices"][0]["delta"]["content"].as_str() {
                if !text.is_empty() {
                    let _ = tx.send(AiEvent::Delta(text.to_string()));
                }
            }
        }
    }

    Ok(())
}
