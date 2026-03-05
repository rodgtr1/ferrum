use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc;
use anyhow::Result;
use std::time::Duration;
use crate::models::response::ResponseData;

/// Maximum response body we'll read into memory (10 MiB).
pub const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    HttpResponse {
        response: ResponseData,
        _request_id: String,
    },
    HttpError {
        error: String,
        _request_id: String,
    },
}

pub fn spawn_event_reader(tx: mpsc::Sender<AppEvent>) {
    tokio::task::spawn_blocking(move || {
        loop {
            match event::read() {
                Ok(CrosstermEvent::Key(key)) => {
                    if tx.blocking_send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}

pub fn spawn_tick(tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(200);
        loop {
            tokio::time::sleep(interval).await;
            if tx.send(AppEvent::Tick).await.is_err() {
                break;
            }
        }
    });
}

/// Task sent to the HTTP worker
#[derive(Debug)]
pub struct HttpTask {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

pub fn spawn_http_worker(
    mut task_rx: mpsc::Receiver<HttpTask>,
    event_tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(false)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        while let Some(task) = task_rx.recv().await {
            let client = client.clone();
            let event_tx = event_tx.clone();

            tokio::spawn(async move {
                let result = execute_request(&client, &task).await;
                match result {
                    Ok(response) => {
                        let _ = event_tx
                            .send(AppEvent::HttpResponse {
                                response,
                                _request_id: task.request_id,
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = event_tx
                            .send(AppEvent::HttpError {
                                error: e.to_string(),
                                _request_id: task.request_id,
                            })
                            .await;
                    }
                }
            });
        }
    });
    Ok(())
}

async fn execute_request(
    client: &reqwest::Client,
    task: &HttpTask,
) -> anyhow::Result<ResponseData> {
    use crate::models::request::KeyValuePair;

    let method = reqwest::Method::from_bytes(task.method.as_bytes())?;
    let mut req = client.request(method, &task.url);

    for (k, v) in &task.headers {
        // Guard against CRLF injection in header values
        if v.contains('\r') || v.contains('\n') {
            return Err(anyhow::anyhow!("Invalid header value for '{}': contains CRLF", k));
        }
        req = req.header(k, v);
    }

    if let Some(body) = &task.body {
        req = req.body(body.clone());
    }

    let start = std::time::Instant::now();
    let resp = req.send().await?;
    let elapsed_ms = start.elapsed().as_millis();

    let status_code = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();

    let resp_headers: Vec<KeyValuePair> = resp
        .headers()
        .iter()
        .map(|(k, v)| KeyValuePair {
            key: k.to_string(),
            value: v.to_str().unwrap_or("").to_string(),
            enabled: true,
        })
        .collect();

    // Cap response body at MAX_BODY_BYTES to prevent memory exhaustion
    let body_bytes = resp.bytes().await?;
    let truncated = body_bytes.len() > MAX_BODY_BYTES;
    let body_slice = if truncated { &body_bytes[..MAX_BODY_BYTES] } else { &body_bytes[..] };
    let size_bytes = body_bytes.len();

    // Avoid the redundant copy from from_utf8_lossy(...).to_string()
    let mut body = String::from_utf8(body_slice.to_vec())
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());

    if truncated {
        body.push_str(&format!("\n\n[… truncated — response exceeded {} MiB]", MAX_BODY_BYTES / 1024 / 1024));
    }

    Ok(ResponseData {
        status_code,
        status_text,
        headers: resp_headers,
        body,
        elapsed_ms,
        size_bytes,
    })
}
