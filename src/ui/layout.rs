use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

pub struct AppLayout {
    pub sidebar: Rect,
    pub request: Rect,
    pub response: Rect,
    pub status_bar: Rect,
}

pub fn compute_layout(area: Rect) -> AppLayout {
    // Outer: status bar at bottom
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let status_bar = outer[1];

    // Horizontal split: sidebar | main
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(outer[0]);

    // Vertical split in main: request (60%) | response (40%)
    let main_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(cols[1]);

    AppLayout {
        sidebar: cols[0],
        request: main_rows[0],
        response: main_rows[1],
        status_bar,
    }
}
