use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use crate::app::{App, FocusPanel, ResponseTab};
use crate::ui::tabs::{response_body_tab, response_headers_tab, response_raw_tab};

pub fn render_response_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPanel::ResponsePanel;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Response ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    render_status_line(f, app, rows[0]);
    render_response_tab_bar(f, app, rows[1]);
    render_response_content(f, app, rows[2]);
}

fn render_status_line(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.loading {
        "⟳ Sending...".to_string()
    } else if let Some(resp) = &app.response {
        let status_color = if resp.status_code < 300 {
            Color::Green
        } else if resp.status_code < 400 {
            Color::Yellow
        } else {
            Color::Red
        };
        let p = Paragraph::new(Line::from(vec![
            ratatui::text::Span::styled(
                format!(" {} ", resp.status_display()),
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
            ratatui::text::Span::raw(format!("  {}ms  •  {}", resp.elapsed_ms, resp.size_display())),
        ]));
        f.render_widget(p, area);
        return;
    } else if let Some(msg) = &app.status_msg {
        msg.clone()
    } else {
        " No response yet — press Enter on URL to send".to_string()
    };

    f.render_widget(Paragraph::new(text).style(Style::default().fg(Color::DarkGray)), area);
}

fn render_response_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPanel::ResponsePanel;
    let tab_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let titles: Vec<Line> = [ResponseTab::Body, ResponseTab::Headers, ResponseTab::Raw]
        .iter()
        .map(|t| Line::from(t.label()))
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.response_tab.index())
        .style(tab_style)
        .highlight_style(Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED));

    f.render_widget(tabs, area);
}

fn render_response_content(f: &mut Frame, app: &App, area: Rect) {
    match app.response_tab {
        ResponseTab::Body => response_body_tab::render(f, app, area),
        ResponseTab::Headers => response_headers_tab::render(f, app, area),
        ResponseTab::Raw => response_raw_tab::render(f, app, area),
    }
}
