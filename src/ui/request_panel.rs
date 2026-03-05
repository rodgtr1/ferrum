use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use crate::app::{App, FocusPanel, RequestTab};
use crate::ui::tabs::{
    auth_tab, body_tab, headers_tab, params_tab,
};

pub fn render_request_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused_main = matches!(
        app.focus,
        FocusPanel::RequestUrl
            | FocusPanel::RequestMethod
            | FocusPanel::RequestTabs
            | FocusPanel::RequestBody
    );

    let border_style = if focused_main {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Request ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split inner: top row (method + url) | tab bar + tab content
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    render_method_url(f, app, rows[0]);
    render_tab_bar(f, app, rows[1]);
    render_tab_content(f, app, rows[2]);
}

fn render_method_url(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    // Method button
    let method_focused = app.focus == FocusPanel::RequestMethod;
    let method_style = if method_focused {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(method_color(&app.edit_method.as_str())).add_modifier(Modifier::BOLD)
    };
    let method_widget = Paragraph::new(format!(" {} ▼", app.edit_method.as_str()))
        .style(method_style);
    f.render_widget(method_widget, cols[0]);

    // URL bar
    let url_focused = app.focus == FocusPanel::RequestUrl;
    let (url_display, url_style) = if url_focused && app.insert_mode {
        (
            format!("{}█", app.edit_url),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        )
    } else if url_focused {
        (
            app.edit_url.clone(),
            Style::default().fg(Color::Yellow),
        )
    } else {
        (
            app.edit_url.clone(),
            Style::default().fg(Color::White),
        )
    };

    let placeholder = if url_display.is_empty() && !url_focused {
        "  Enter URL... (Tab to focus, i to edit)"
    } else {
        ""
    };

    let display = if url_display.is_empty() {
        placeholder.to_string()
    } else {
        format!(" {}", url_display)
    };

    let url_widget = Paragraph::new(display).style(url_style);
    f.render_widget(url_widget, cols[1]);
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tab_focused = matches!(app.focus, FocusPanel::RequestTabs | FocusPanel::RequestBody);
    let tab_style = if tab_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let titles: Vec<Line> = [RequestTab::Headers, RequestTab::Body, RequestTab::Params, RequestTab::Auth]
        .iter()
        .map(|t| Line::from(t.label()))
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.request_tab.index())
        .style(tab_style)
        .highlight_style(Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED));

    f.render_widget(tabs, area);
}

fn render_tab_content(f: &mut Frame, app: &App, area: Rect) {
    match app.request_tab {
        RequestTab::Headers => headers_tab::render(f, app, area),
        RequestTab::Body => body_tab::render(f, app, area),
        RequestTab::Params => params_tab::render(f, app, area),
        RequestTab::Auth => auth_tab::render(f, app, area),
    }
}

fn method_color(method: &str) -> Color {
    match method {
        "GET" => Color::Green,
        "POST" => Color::Blue,
        "PUT" => Color::Yellow,
        "PATCH" => Color::Magenta,
        "DELETE" => Color::Red,
        _ => Color::Cyan,
    }
}
