use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table, TableState},
    Frame,
};
use crate::app::{App, FocusPanel};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let focused = matches!(app.focus, FocusPanel::RequestTabs | FocusPanel::RequestBody);
    let kv = &app.params_kv;

    let rows: Vec<Row> = app
        .edit_params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_editing = kv.editing_col.is_some() && kv.selected == i;
            let key_val = if is_editing && kv.editing_col == Some(0) {
                format!("{}█", kv.edit_buf)
            } else {
                p.key.clone()
            };
            let val_val = if is_editing && kv.editing_col == Some(1) {
                format!("{}█", kv.edit_buf)
            } else {
                p.value.clone()
            };

            let enabled_marker = if p.enabled { "✓" } else { " " };
            let row_style = if p.enabled {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(enabled_marker).style(Style::default().fg(Color::Green)),
                Cell::from(key_val).style(Style::default().fg(Color::Cyan)),
                Cell::from(val_val),
            ])
            .style(row_style)
        })
        .collect();

    let mut state = TableState::default();
    if !app.edit_params.is_empty() && focused {
        state.select(Some(kv.selected));
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(40),
            Constraint::Min(0),
        ],
    )
    .header(Row::new(vec!["", "Key", "Value"]).style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    ))
    .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
    .column_spacing(1);

    f.render_stateful_widget(table, area, &mut state);

    if app.edit_params.is_empty() {
        let hint = ratatui::widgets::Paragraph::new(" o  add param   dd  delete   Space  toggle")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, area);
    }
}
