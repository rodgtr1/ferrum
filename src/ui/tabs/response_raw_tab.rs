use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if let Some(resp) = &app.response {
        let raw = resp.body.as_str();
        let widget = Paragraph::new(raw)
            .style(Style::default().fg(Color::White))
            .scroll((app.response_scroll, 0))
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(widget, area);
    } else {
        let hint = Paragraph::new(" No raw response")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, area);
    }
}
