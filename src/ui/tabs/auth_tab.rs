use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;
use crate::models::request::AuthConfig;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    let sel = |active: bool| -> Style {
        if active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    let none_active = matches!(app.edit_auth, AuthConfig::None);
    let bearer_active = matches!(app.edit_auth, AuthConfig::BearerToken { .. });
    let basic_active = matches!(app.edit_auth, AuthConfig::BasicAuth { .. });
    let apikey_active = matches!(app.edit_auth, AuthConfig::ApiKey { .. });

    lines.push(Line::from(vec![
        Span::styled("[1] No Auth  ", sel(none_active)),
        Span::styled("[2] Bearer Token  ", sel(bearer_active)),
        Span::styled("[3] Basic Auth  ", sel(basic_active)),
        Span::styled("[4] API Key", sel(apikey_active)),
    ]));

    lines.push(Line::from(""));

    match &app.edit_auth {
        AuthConfig::None => {
            lines.push(Line::from(Span::styled(
                " No authentication",
                Style::default().fg(Color::DarkGray),
            )));
        }
        AuthConfig::BearerToken { token } => {
            lines.push(Line::from(vec![
                Span::styled(" Token: ", Style::default().fg(Color::Yellow)),
                Span::raw("*".repeat(token.len().min(40))),
            ]));
        }
        AuthConfig::BasicAuth { username, password } => {
            lines.push(Line::from(vec![
                Span::styled(" Username: ", Style::default().fg(Color::Yellow)),
                Span::raw(username),
            ]));
            lines.push(Line::from(vec![
                Span::styled(" Password: ", Style::default().fg(Color::Yellow)),
                Span::raw("*".repeat(password.len())),
            ]));
        }
        AuthConfig::ApiKey { header, value } => {
            lines.push(Line::from(vec![
                Span::styled(" Header: ", Style::default().fg(Color::Yellow)),
                Span::raw(header),
            ]));
            lines.push(Line::from(vec![
                Span::styled(" Value: ", Style::default().fg(Color::Yellow)),
                Span::raw("*".repeat(value.len().min(40))),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}
