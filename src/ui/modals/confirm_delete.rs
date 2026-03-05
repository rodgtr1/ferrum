use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, label: &str, area: Rect) {
    let modal_area = centered_rect(50, 7, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Confirm Delete ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new(format!("Delete: {}", label))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        rows[0],
    );
    f.render_widget(
        Paragraph::new("This action cannot be undone.")
            .style(Style::default().fg(Color::Red)),
        rows[1],
    );
    f.render_widget(
        Paragraph::new("y / Enter to confirm • n / Esc to cancel")
            .style(Style::default().fg(Color::DarkGray)),
        rows[2],
    );
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
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
