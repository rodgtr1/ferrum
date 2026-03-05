use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect) {
    let modal_area = centered_rect(65, 80, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Help — ferrum ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let key = |k: &str| Span::styled(format!("{:<14}", k), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    let desc = |d: &str| Span::styled(d.to_string(), Style::default().fg(Color::White));

    let rows: Vec<Line> = vec![
        Line::from(Span::styled("Global", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("Tab / S-Tab"), desc("Cycle focus panels")]),
        Line::from(vec![key("q / Ctrl-C"), desc("Quit")]),
        Line::from(vec![key("?"), desc("Toggle this help")]),
        Line::from(vec![key("e"), desc("Environment editor")]),
        Line::from(""),
        Line::from(Span::styled("Sidebar", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("j / k"), desc("Navigate up/down")]),
        Line::from(vec![key("l / Enter"), desc("Expand collection / open request")]),
        Line::from(vec![key("h"), desc("Collapse collection")]),
        Line::from(vec![key("n"), desc("New collection")]),
        Line::from(vec![key("N"), desc("New request in collection")]),
        Line::from(vec![key("dd"), desc("Delete selected item")]),
        Line::from(""),
        Line::from(Span::styled("Request", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("i"), desc("Enter insert mode (URL / Body)")]),
        Line::from(vec![key("Esc"), desc("Exit insert mode, save")]),
        Line::from(vec![key("Enter"), desc("Send request (from URL bar)")]),
        Line::from(vec![key("m"), desc("Focus method selector")]),
        Line::from(vec![key("[ / ]"), desc("Prev / next request tab")]),
        Line::from(vec![key("1 2 3 4"), desc("Jump to Headers/Body/Params/Auth")]),
        Line::from(""),
        Line::from(Span::styled("KV Tables (Headers/Params)", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("o"), desc("Add row")]),
        Line::from(vec![key("dd"), desc("Delete row")]),
        Line::from(vec![key("Space"), desc("Toggle enabled")]),
        Line::from(vec![key("Enter"), desc("Edit row")]),
        Line::from(""),
        Line::from(Span::styled("Response", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![key("j / k"), desc("Scroll body")]),
        Line::from(vec![key("[ / ]"), desc("Switch response tab")]),
        Line::from(""),
        Line::from(Span::styled("? / Esc / q  — close this help", Style::default().fg(Color::DarkGray))),
    ];

    f.render_widget(Paragraph::new(rows), inner);
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
