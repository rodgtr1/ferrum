use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if let Some(resp) = &app.response {
        let is_json = resp.headers.iter().any(|h| {
            h.key.to_lowercase() == "content-type"
                && h.value.to_lowercase().contains("json")
        }) || resp.body.trim_start().starts_with('{')
            || resp.body.trim_start().starts_with('[');

        if is_json && !resp.body.is_empty() {
            // Use the pre-computed highlight cache — never re-highlight on render
            if let Some(lines) = &app.response_highlighted {
                let widget = Paragraph::new(Text::from(lines.clone()))
                    .scroll((app.response_scroll, 0));
                f.render_widget(widget, area);
            } else {
                // Fallback: plain text until cache is populated
                let widget = Paragraph::new(resp.body.as_str())
                    .style(Style::default().fg(Color::White))
                    .scroll((app.response_scroll, 0))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                f.render_widget(widget, area);
            }
        } else {
            let widget = Paragraph::new(resp.body.as_str())
                .style(Style::default().fg(Color::White))
                .scroll((app.response_scroll, 0))
                .wrap(ratatui::widgets::Wrap { trim: false });
            f.render_widget(widget, area);
        }
    } else {
        let hint = Paragraph::new(" No response body")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, area);
    }
}
