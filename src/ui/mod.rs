pub mod highlight;
pub mod layout;
pub mod modals;
pub mod request_panel;
pub mod response_panel;
pub mod sidebar;
pub mod tabs;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{ActiveModal, App, DeleteTarget};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let lay = layout::compute_layout(area);

    sidebar::render_sidebar(f, app, lay.sidebar);
    request_panel::render_request_panel(f, app, lay.request);
    response_panel::render_response_panel(f, app, lay.response);
    render_status_bar(f, app, lay.status_bar);
    render_modal(f, app, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let focus_label = match &app.focus {
        crate::app::FocusPanel::Sidebar => "SIDEBAR",
        crate::app::FocusPanel::RequestUrl => "URL",
        crate::app::FocusPanel::RequestMethod => "METHOD",
        crate::app::FocusPanel::RequestTabs => "REQUEST",
        crate::app::FocusPanel::RequestBody => "BODY",
        crate::app::FocusPanel::ResponsePanel => "RESPONSE",
    };

    let mode = if app.insert_mode { " INSERT " } else { " NORMAL " };

    let env_label = app
        .active_env_idx
        .and_then(|i| app.environments.get(i))
        .map(|e| format!(" env:{} ", e.name))
        .unwrap_or_else(|| " no-env ".to_string());

    let line = Line::from(vec![
        Span::styled(mode, Style::default().fg(Color::Black).bg(if app.insert_mode { Color::Yellow } else { Color::Green }).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(focus_label, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(env_label, Style::default().fg(Color::Magenta)),
        Span::raw("  "),
        Span::styled("? help  q quit  Tab focus  e env", Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}

fn render_modal(f: &mut Frame, app: &App, area: Rect) {
    match &app.modal {
        ActiveModal::None => {}
        ActiveModal::NewCollection { name } => {
            modals::new_collection::render(f, name, area);
        }
        ActiveModal::NewRequest { name, collection_idx } => {
            let col_name = app
                .collections
                .get(*collection_idx)
                .map(|c| c.name.as_str())
                .unwrap_or("?");
            modals::new_request::render(f, name, col_name, area);
        }
        ActiveModal::ConfirmDelete { target } => {
            let label = match target {
                DeleteTarget::Collection(ci) => app
                    .collections
                    .get(*ci)
                    .map(|c| format!("collection \"{}\"", c.name))
                    .unwrap_or_else(|| "item".to_string()),
                DeleteTarget::Request { collection_idx, request_idx } => app
                    .collections
                    .get(*collection_idx)
                    .and_then(|c| c.requests.get(*request_idx))
                    .map(|r| format!("request \"{}\"", r.name))
                    .unwrap_or_else(|| "request".to_string()),
            };
            modals::confirm_delete::render(f, &label, area);
        }
        ActiveModal::Environment => {
            modals::environment::render(f, app, area);
        }
        ActiveModal::Help => {
            modals::help::render(f, area);
        }
    }
}
