mod app;
mod event;
mod http;
mod models;
mod storage;
mod tui;
mod ui;
mod utils;

use anyhow::Result;
use app::App;
use event::{spawn_event_reader, spawn_http_worker, spawn_tick, AppEvent};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);
    let (http_tx, http_rx) = mpsc::channel::<event::HttpTask>(64);

    spawn_event_reader(event_tx.clone());
    spawn_tick(event_tx.clone());
    spawn_http_worker(http_rx, event_tx.clone())?;

    let mut app = App::new(http_tx)?;
    let mut terminal = tui::init()?;

    terminal.draw(|f| ui::render(f, &app))?;

    loop {
        let Some(event) = event_rx.recv().await else {
            break;
        };

        let should_quit = app.handle_event(event);
        if should_quit {
            break;
        }

        terminal.draw(|f| ui::render(f, &app))?;
    }

    tui::restore()?;
    Ok(())
}
