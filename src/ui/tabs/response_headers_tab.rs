use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table},
    Frame,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if let Some(resp) = &app.response {
        let rows: Vec<Row> = resp
            .headers
            .iter()
            .map(|h| {
                Row::new(vec![
                    Cell::from(h.key.as_str()).style(Style::default().fg(Color::Cyan)),
                    Cell::from(h.value.as_str()),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(35), Constraint::Min(0)],
        )
        .header(
            Row::new(vec!["Header", "Value"]).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .column_spacing(1);

        f.render_widget(table, area);
    } else {
        let hint = ratatui::widgets::Paragraph::new(" No response headers")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, area);
    }
}
