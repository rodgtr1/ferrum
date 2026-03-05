use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Row, Table, TableState},
    Frame,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let modal_area = centered_rect(80, 70, area);
    f.render_widget(Clear, modal_area);

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Environments  (n new  o add var  J/K var nav  a set active  Esc close) ");
    let inner = outer_block.inner(modal_area);
    f.render_widget(outer_block, modal_area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(inner);

    // Left: env list
    let items: Vec<ListItem> = app
        .environments
        .iter()
        .enumerate()
        .map(|(i, env)| {
            let is_active = app.active_env_idx == Some(i);
            let marker = if is_active { "● " } else { "  " };
            ListItem::new(format!("{}{}", marker, env.name))
        })
        .collect();

    let mut env_state = ListState::default();
    if !app.environments.is_empty() {
        env_state.select(Some(app.env_list_cursor));
    }

    let env_list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title("Envs"))
        .highlight_style(Style::default().bg(Color::Blue));

    f.render_stateful_widget(env_list, cols[0], &mut env_state);

    // Right: var table
    if let Some(env) = app.environments.get(app.env_list_cursor) {
        let rows: Vec<Row> = env
            .vars
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let is_editing = app.env_editing.map(|(ei, vi)| ei == app.env_list_cursor && vi / 2 == i);
                let col = app.env_editing.map(|(_, vi)| vi % 2);

                let key_display = if is_editing == Some(true) && col == Some(0) {
                    format!("{}█", app.env_edit_buf)
                } else {
                    v.key.clone()
                };
                let val_display = if is_editing == Some(true) && col == Some(1) {
                    format!("{}█", app.env_edit_buf)
                } else {
                    v.value.clone()
                };

                Row::new(vec![
                    Cell::from(key_display).style(Style::default().fg(Color::Cyan)),
                    Cell::from(val_display),
                ])
            })
            .collect();

        let mut var_state = TableState::default();
        if !env.vars.is_empty() {
            var_state.select(Some(app.env_var_cursor));
        }

        let var_table = Table::new(
            rows,
            [Constraint::Percentage(40), Constraint::Min(0)],
        )
        .header(
            Row::new(vec!["Key", "Value"]).style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(Block::default().title(format!(" {} ", env.name)))
        .row_highlight_style(Style::default().bg(Color::DarkGray))
        .column_spacing(1);

        f.render_stateful_widget(var_table, cols[1], &mut var_state);
    } else {
        f.render_widget(
            ratatui::widgets::Paragraph::new(" No environments yet. Press n to create one.")
                .style(Style::default().fg(Color::DarkGray)),
            cols[1],
        );
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
