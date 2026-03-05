use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::app::{App, FocusPanel, SidebarItem};

pub fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == FocusPanel::Sidebar;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .map(|item| match item {
            SidebarItem::Collection(ci) => {
                let col = &app.collections[*ci];
                let arrow = if col.expanded { "▾ " } else { "▸ " };
                let text = format!("{}{}", arrow, col.name);
                ListItem::new(Line::from(vec![Span::styled(
                    text,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]))
            }
            SidebarItem::Request { col, req } => {
                let req_item = &app.collections[*col].requests[*req];
                let method_color = method_color(req_item.config.method.as_str());
                let is_selected = app.selected_collection == Some(*col)
                    && app.selected_request == Some(*req);

                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        req_item.config.method.as_str(),
                        Style::default().fg(method_color),
                    ),
                    Span::raw(" "),
                    Span::styled(req_item.name.as_str(), style),
                ]))
            }
        })
        .collect();

    let mut state = ListState::default();
    if !app.sidebar_items.is_empty() {
        state.select(Some(app.sidebar_cursor));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Collections "),
        )
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_stateful_widget(list, area, &mut state);
}

fn method_color(method: &str) -> Color {
    match method {
        "GET" => Color::Green,
        "POST" => Color::Blue,
        "PUT" => Color::Yellow,
        "PATCH" => Color::Magenta,
        "DELETE" => Color::Red,
        "HEAD" | "OPTIONS" => Color::Cyan,
        _ => Color::White,
    }
}
