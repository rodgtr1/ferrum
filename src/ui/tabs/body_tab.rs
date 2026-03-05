use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, FocusPanel};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let focused = matches!(app.focus, FocusPanel::RequestBody);
    let style = if focused && app.insert_mode {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let display = if app.edit_body.is_empty() {
        if focused {
            "█".to_string()
        } else {
            " i  to enter insert mode and type body...".to_string()
        }
    } else if focused && app.insert_mode {
        format!("{}█", app.edit_body)
    } else {
        app.edit_body.clone()
    };

    let widget = Paragraph::new(display).style(style).wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(widget, area);
}
