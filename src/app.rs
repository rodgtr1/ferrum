use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use uuid::Uuid;

use ratatui::text::Line;

use crate::{
    event::{AppEvent, HttpTask},
    models::{
        collection::{Collection, RequestItem},
        environment::Environment,
        history::HistoryEntry,
        request::{AuthConfig, HttpMethod, KeyValuePair},
        response::ResponseData,
    },
    storage,
    utils::{env_interpolation::interpolate, json_format::pretty_print},
};

// ── Focus / Modal enums ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FocusPanel {
    #[default]
    Sidebar,
    RequestUrl,
    RequestMethod,
    RequestTabs,
    RequestBody,
    ResponsePanel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestTab {
    Headers,
    Body,
    Params,
    Auth,
}

impl RequestTab {
    pub fn index(&self) -> usize {
        match self {
            RequestTab::Headers => 0,
            RequestTab::Body => 1,
            RequestTab::Params => 2,
            RequestTab::Auth => 3,
        }
    }
    pub fn from_index(i: usize) -> Self {
        match i % 4 {
            0 => RequestTab::Headers,
            1 => RequestTab::Body,
            2 => RequestTab::Params,
            _ => RequestTab::Auth,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            RequestTab::Headers => "Headers",
            RequestTab::Body => "Body",
            RequestTab::Params => "Params",
            RequestTab::Auth => "Auth",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseTab {
    Body,
    Headers,
    Raw,
}

impl ResponseTab {
    pub fn index(&self) -> usize {
        match self {
            ResponseTab::Body => 0,
            ResponseTab::Headers => 1,
            ResponseTab::Raw => 2,
        }
    }
    pub fn from_index(i: usize) -> Self {
        match i % 3 {
            0 => ResponseTab::Body,
            1 => ResponseTab::Headers,
            _ => ResponseTab::Raw,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            ResponseTab::Body => "Body",
            ResponseTab::Headers => "Headers",
            ResponseTab::Raw => "Raw",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveModal {
    None,
    NewCollection { name: String },
    NewRequest { name: String, collection_idx: usize },
    Environment,
    ConfirmDelete { target: DeleteTarget },
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteTarget {
    Collection(usize),
    Request { collection_idx: usize, request_idx: usize },
}

// ── KV table editing state ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct KvTableState {
    pub selected: usize,
    pub editing_col: Option<usize>, // 0=key, 1=value
    pub edit_buf: String,
    pub pending_dd: bool,
}

impl KvTableState {
    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.selected = 0;
        } else if self.selected >= len {
            self.selected = len - 1;
        }
    }
}

// ── Sidebar cursor ───────────────────────────────────────────────────────────

/// Flat list item representing either a Collection header or a nested Request
#[derive(Debug, Clone)]
pub enum SidebarItem {
    Collection(usize),
    Request { col: usize, req: usize },
}

// ── Main App State ───────────────────────────────────────────────────────────

pub struct App {
    // Data
    pub collections: Vec<Collection>,
    pub environments: Vec<Environment>,
    pub active_env_idx: Option<usize>,
    pub history: Vec<HistoryEntry>,

    // Sidebar
    pub sidebar_cursor: usize, // index into flat list
    pub sidebar_items: Vec<SidebarItem>, // rebuilt on change

    // Selected request (collection, request within)
    pub selected_collection: Option<usize>,
    pub selected_request: Option<usize>,

    // Active request editor state (mirrors selected RequestItem or scratch)
    pub edit_url: String,
    pub edit_method: HttpMethod,
    pub edit_headers: Vec<KeyValuePair>,
    pub edit_params: Vec<KeyValuePair>,
    pub edit_body: String,
    pub edit_auth: AuthConfig,

    // Focus / tabs
    pub focus: FocusPanel,
    pub request_tab: RequestTab,
    pub response_tab: ResponseTab,

    // KV table states
    pub headers_kv: KvTableState,
    pub params_kv: KvTableState,

    // Response
    pub response: Option<ResponseData>,
    pub response_highlighted: Option<Vec<Line<'static>>>,
    pub response_scroll: u16,
    pub loading: bool,
    pub status_msg: Option<String>,

    // Modal
    pub modal: ActiveModal,

    // Env editor state
    pub env_list_cursor: usize,
    pub env_var_cursor: usize,
    pub env_editing: Option<(usize, usize)>, // (env_idx, var_idx) col 0/1
    pub env_edit_buf: String,

    // HTTP channel
    pub http_tx: mpsc::Sender<HttpTask>,

    // Pending delete key sequence
    pub dd_count: u8,

    // Insert mode for text fields
    pub insert_mode: bool,
}

impl App {
    pub fn new(http_tx: mpsc::Sender<HttpTask>) -> Result<Self> {
        let collections = storage::collections::load_all().unwrap_or_default();
        let environments = storage::environments::load_all().unwrap_or_default();
        let history = storage::history::load().unwrap_or_default();

        let mut app = Self {
            collections,
            environments,
            active_env_idx: None,
            history,
            sidebar_cursor: 0,
            sidebar_items: Vec::new(),
            selected_collection: None,
            selected_request: None,
            edit_url: String::new(),
            edit_method: HttpMethod::GET,
            edit_headers: Vec::new(),
            edit_params: Vec::new(),
            edit_body: String::new(),
            edit_auth: AuthConfig::None,
            focus: FocusPanel::Sidebar,
            request_tab: RequestTab::Headers,
            response_tab: ResponseTab::Body,
            headers_kv: KvTableState::default(),
            params_kv: KvTableState::default(),
            response: None,
            response_highlighted: None,
            response_scroll: 0,
            loading: false,
            status_msg: None,
            modal: ActiveModal::None,
            env_list_cursor: 0,
            env_var_cursor: 0,
            env_editing: None,
            env_edit_buf: String::new(),
            http_tx,
            dd_count: 0,
            insert_mode: false,
        };
        app.rebuild_sidebar();
        Ok(app)
    }

    // ── Sidebar helpers ──────────────────────────────────────────────────────

    pub fn rebuild_sidebar(&mut self) {
        self.sidebar_items.clear();
        for (ci, col) in self.collections.iter().enumerate() {
            self.sidebar_items.push(SidebarItem::Collection(ci));
            if col.expanded {
                for ri in 0..col.requests.len() {
                    self.sidebar_items.push(SidebarItem::Request { col: ci, req: ri });
                }
            }
        }
        let len = self.sidebar_items.len();
        if self.sidebar_cursor >= len && len > 0 {
            self.sidebar_cursor = len - 1;
        }
    }

    fn sidebar_item_at_cursor(&self) -> Option<SidebarItem> {
        self.sidebar_items.get(self.sidebar_cursor).cloned()
    }

    fn load_selected_request(&mut self) {
        if let (Some(ci), Some(ri)) = (self.selected_collection, self.selected_request) {
            if let Some(col) = self.collections.get(ci) {
                if let Some(req) = col.requests.get(ri) {
                    self.edit_url = req.config.url.clone();
                    self.edit_method = req.config.method.clone();
                    self.edit_headers = req.config.headers.clone();
                    self.edit_params = req.config.query_params.clone();
                    self.edit_body = req.config.body.clone();
                    self.edit_auth = req.config.auth.clone();
                    self.response = None;
                    self.response_highlighted = None;
                    self.response_scroll = 0;
                }
            }
        }
    }

    fn save_to_selected_request(&mut self) {
        if let (Some(ci), Some(ri)) = (self.selected_collection, self.selected_request) {
            if let Some(col) = self.collections.get_mut(ci) {
                if let Some(req) = col.requests.get_mut(ri) {
                    req.config.url = self.edit_url.clone();
                    req.config.method = self.edit_method.clone();
                    req.config.headers = self.edit_headers.clone();
                    req.config.query_params = self.edit_params.clone();
                    req.config.body = self.edit_body.clone();
                    req.config.auth = self.edit_auth.clone();
                }
                let _ = storage::collections::save(col);
            }
        }
    }

    // ── HTTP send ────────────────────────────────────────────────────────────

    pub fn send_request(&mut self) {
        if self.edit_url.is_empty() {
            self.status_msg = Some("URL is empty".to_string());
            return;
        }

        let active_env = self.active_env_idx.and_then(|i| self.environments.get(i));
        let url = interpolate(&self.edit_url, active_env);

        let mut headers: Vec<(String, String)> = self
            .edit_headers
            .iter()
            .filter(|h| h.enabled && !h.key.is_empty())
            .map(|h| {
                (
                    interpolate(&h.key, active_env),
                    interpolate(&h.value, active_env),
                )
            })
            .collect();

        // Add auth header
        match &self.edit_auth {
            AuthConfig::BearerToken { token } => {
                headers.push(("Authorization".into(), format!("Bearer {}", interpolate(token, active_env))));
            }
            AuthConfig::BasicAuth { username, password } => {
                let creds = format!("{}:{}", interpolate(username, active_env), interpolate(password, active_env));
                let encoded = base64_encode(creds.as_bytes());
                headers.push(("Authorization".into(), format!("Basic {}", encoded)));
            }
            AuthConfig::ApiKey { header, value } => {
                headers.push((header.clone(), interpolate(value, active_env)));
            }
            AuthConfig::None => {}
        }

        // Add query params to URL
        let params: Vec<(String, String)> = self
            .edit_params
            .iter()
            .filter(|p| p.enabled && !p.key.is_empty())
            .map(|p| {
                (
                    interpolate(&p.key, active_env),
                    interpolate(&p.value, active_env),
                )
            })
            .collect();

        let final_url = if params.is_empty() {
            url
        } else {
            // Percent-encode keys and values to prevent parameter injection
            let qs: String = params
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        percent_encode(k),
                        percent_encode(v)
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            if url.contains('?') {
                format!("{}&{}", url, qs)
            } else {
                format!("{}?{}", url, qs)
            }
        };

        let body = if self.edit_body.is_empty() {
            None
        } else {
            Some(interpolate(&self.edit_body, active_env))
        };

        let request_id = Uuid::new_v4().to_string();
        let task = HttpTask {
            request_id,
            method: self.edit_method.as_str().to_string(),
            url: final_url,
            headers,
            body,
        };

        let _ = self.http_tx.try_send(task);
        self.loading = true;
        self.response = None;
        self.response_highlighted = None;
        self.status_msg = Some("Sending...".to_string());
    }

    // ── Event handler ─────────────────────────────────────────────────────────

    pub fn handle_event(&mut self, event: AppEvent) -> bool {
        match event {
            AppEvent::Tick => {
                // nothing to update on tick currently
                false
            }
            AppEvent::Key(key) => self.handle_key(key),
            AppEvent::HttpResponse { mut response, .. } => {
                self.loading = false;
                self.status_msg = None;
                // Pretty-print JSON body in place — no clone of ResponseData
                response.body = pretty_print(&response.body);
                // Log sanitized entry to history (no auth credentials)
                let entry = HistoryEntry::from_request(
                    &self.edit_url,
                    &self.edit_method,
                    &self.edit_headers,
                    &self.edit_params,
                ).with_response(&response);
                self.history.push(entry);
                // Persist history asynchronously to avoid blocking the render loop
                let history_snapshot = self.history.clone();
                tokio::spawn(async move {
                    let _ = storage::history::save(&history_snapshot);
                });
                // Build highlight cache once here; render reads from cache
                self.response_highlighted = Some(
                    crate::ui::highlight::Highlighter::new().highlight_json(&response.body)
                );
                self.response = Some(response);
                self.response_scroll = 0;
                false
            }
            AppEvent::HttpError { error, .. } => {
                self.loading = false;
                self.status_msg = Some(format!("Error: {}", error));
                let entry = HistoryEntry::from_request(
                    &self.edit_url,
                    &self.edit_method,
                    &self.edit_headers,
                    &self.edit_params,
                ).with_error(error);
                self.history.push(entry);
                let history_snapshot = self.history.clone();
                tokio::spawn(async move {
                    let _ = storage::history::save(&history_snapshot);
                });
                false
            }
        }
    }

    fn is_kv_editing(&self) -> bool {
        self.focus == FocusPanel::RequestTabs
            && match self.request_tab {
                RequestTab::Headers => self.headers_kv.editing_col.is_some(),
                RequestTab::Params => self.params_kv.editing_col.is_some(),
                _ => false,
            }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Global quit
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return true;
        }

        // Modal handling takes priority
        if self.modal != ActiveModal::None {
            return self.handle_modal_key(key);
        }

        // Insert mode for URL / body
        if self.insert_mode {
            return self.handle_insert_key(key);
        }

        match key.code {
            KeyCode::Char('q') if !self.is_kv_editing() => return true,
            KeyCode::Tab if !self.is_kv_editing() => self.cycle_focus(1),
            KeyCode::BackTab if !self.is_kv_editing() => self.cycle_focus(-1),
            KeyCode::Char('?') if !self.is_kv_editing() => self.modal = ActiveModal::Help,
            KeyCode::Char('e') if !self.is_kv_editing() => self.modal = ActiveModal::Environment,
            _ => match self.focus.clone() {
                FocusPanel::Sidebar => self.handle_sidebar_key(key),
                FocusPanel::RequestUrl => self.handle_url_key(key),
                FocusPanel::RequestMethod => self.handle_method_key(key),
                FocusPanel::RequestTabs => self.handle_request_tabs_key(key),
                FocusPanel::RequestBody => self.handle_request_body_key(key),
                FocusPanel::ResponsePanel => self.handle_response_key(key),
            },
        }
        false
    }

    fn cycle_focus(&mut self, dir: i8) {
        let panels = [
            FocusPanel::Sidebar,
            FocusPanel::RequestUrl,
            FocusPanel::RequestTabs,
            FocusPanel::ResponsePanel,
        ];
        let current = panels.iter().position(|p| p == &self.focus).unwrap_or(0);
        let next = if dir > 0 {
            (current + 1) % panels.len()
        } else {
            (current + panels.len() - 1) % panels.len()
        };
        self.focus = panels[next].clone();
        self.insert_mode = false;
    }

    // ── Sidebar key handler ──────────────────────────────────────────────────

    fn handle_sidebar_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.sidebar_items.len();
                if len > 0 && self.sidebar_cursor < len - 1 {
                    self.sidebar_cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.sidebar_cursor > 0 {
                    self.sidebar_cursor -= 1;
                }
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                if let Some(item) = self.sidebar_item_at_cursor() {
                    match item {
                        SidebarItem::Collection(ci) => {
                            self.collections[ci].expanded = !self.collections[ci].expanded;
                            self.rebuild_sidebar();
                        }
                        SidebarItem::Request { col, req } => {
                            self.selected_collection = Some(col);
                            self.selected_request = Some(req);
                            self.load_selected_request();
                            self.focus = FocusPanel::RequestUrl;
                        }
                    }
                }
            }
            KeyCode::Char('h') => {
                if let Some(SidebarItem::Collection(ci)) = self.sidebar_item_at_cursor() {
                    self.collections[ci].expanded = false;
                    self.rebuild_sidebar();
                }
            }
            KeyCode::Char('n') => {
                self.modal = ActiveModal::NewCollection { name: String::new() };
            }
            KeyCode::Char('N') => {
                // New request in current collection
                if let Some(SidebarItem::Collection(ci)) = self.sidebar_item_at_cursor() {
                    self.modal = ActiveModal::NewRequest { name: String::new(), collection_idx: ci };
                } else if let Some(SidebarItem::Request { col, .. }) = self.sidebar_item_at_cursor() {
                    self.modal = ActiveModal::NewRequest { name: String::new(), collection_idx: col };
                } else if !self.collections.is_empty() {
                    self.modal = ActiveModal::NewRequest { name: String::new(), collection_idx: 0 };
                }
            }
            KeyCode::Char('d') => {
                self.dd_count += 1;
                if self.dd_count >= 2 {
                    self.dd_count = 0;
                    self.status_msg = None;
                    if let Some(item) = self.sidebar_item_at_cursor() {
                        let target = match item {
                            SidebarItem::Collection(ci) => DeleteTarget::Collection(ci),
                            SidebarItem::Request { col, req } => {
                                DeleteTarget::Request { collection_idx: col, request_idx: req }
                            }
                        };
                        self.modal = ActiveModal::ConfirmDelete { target };
                    }
                } else {
                    self.status_msg = Some("d again to delete".to_string());
                }
            }
            _ => {
                self.dd_count = 0;
                self.status_msg = None;
            }
        }
    }

    // ── URL key handler ──────────────────────────────────────────────────────

    fn handle_url_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('i') => {
                self.insert_mode = true;
            }
            KeyCode::Enter => {
                self.save_to_selected_request();
                self.send_request();
            }
            KeyCode::Char('m') => {
                self.focus = FocusPanel::RequestMethod;
            }
            _ => {}
        }
    }

    // ── Method key handler ───────────────────────────────────────────────────

    fn handle_method_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.edit_method = self.edit_method.cycle_next();
                self.save_to_selected_request();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.edit_method = self.edit_method.cycle_prev();
                self.save_to_selected_request();
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.focus = FocusPanel::RequestUrl;
            }
            _ => {}
        }
    }

    // ── Request tabs key handler ─────────────────────────────────────────────

    fn handle_request_tabs_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('[') => {
                let idx = self.request_tab.index();
                self.request_tab = RequestTab::from_index((idx + 3) % 4);
            }
            KeyCode::Char(']') => {
                let idx = self.request_tab.index();
                self.request_tab = RequestTab::from_index((idx + 1) % 4);
            }
            KeyCode::Char('1') => self.request_tab = RequestTab::Headers,
            KeyCode::Char('2') => self.request_tab = RequestTab::Body,
            KeyCode::Char('3') => self.request_tab = RequestTab::Params,
            KeyCode::Char('4') => self.request_tab = RequestTab::Auth,
            KeyCode::Enter if !self.is_kv_editing() => {
                self.focus = FocusPanel::RequestBody;
            }
            _ => match self.request_tab {
                RequestTab::Headers => self.handle_kv_key(key, true),
                RequestTab::Params => self.handle_kv_key(key, false),
                RequestTab::Body => {
                    if key.code == KeyCode::Char('i') {
                        self.focus = FocusPanel::RequestBody;
                        self.insert_mode = true;
                    }
                }
                RequestTab::Auth => self.handle_auth_key(key),
            },
        }
    }

    fn handle_kv_key(&mut self, key: KeyEvent, is_headers: bool) {
        let (kv, rows) = if is_headers {
            (&mut self.headers_kv, self.edit_headers.len())
        } else {
            (&mut self.params_kv, self.edit_params.len())
        };

        if let Some(col) = kv.editing_col {
            match key.code {
                KeyCode::Char(c) => kv.edit_buf.push(c),
                KeyCode::Backspace => { kv.edit_buf.pop(); }
                KeyCode::Tab | KeyCode::Enter => {
                    // commit edit
                    let idx = kv.selected;
                    let buf = kv.edit_buf.clone();
                    let rows_ref = if is_headers {
                        &mut self.edit_headers
                    } else {
                        &mut self.edit_params
                    };
                    if let Some(row) = rows_ref.get_mut(idx) {
                        if col == 0 { row.key = buf; } else { row.value = buf; }
                    }
                    let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                    kv.edit_buf.clear();
                    kv.editing_col = if col == 0 { Some(1) } else { None };
                    self.save_to_selected_request();
                }
                KeyCode::Esc => {
                    let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                    kv.editing_col = None;
                    kv.edit_buf.clear();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                if kv.selected + 1 < rows { kv.selected += 1; }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                if kv.selected > 0 { kv.selected -= 1; }
            }
            KeyCode::Char('o') => {
                let rows_ref = if is_headers { &mut self.edit_headers } else { &mut self.edit_params };
                rows_ref.push(KeyValuePair::new("", ""));
                let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                kv.selected = rows_ref.len() - 1;
                kv.editing_col = Some(0);
                kv.edit_buf.clear();
            }
            KeyCode::Char('d') => {
                let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                if kv.pending_dd {
                    kv.pending_dd = false;
                    let idx = kv.selected;
                    let rows_ref = if is_headers { &mut self.edit_headers } else { &mut self.edit_params };
                    if idx < rows_ref.len() {
                        rows_ref.remove(idx);
                        let len = rows_ref.len();
                        let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                        kv.clamp(len);
                    }
                    self.save_to_selected_request();
                } else {
                    let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                    kv.pending_dd = true;
                }
            }
            KeyCode::Char(' ') => {
                let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                let idx = kv.selected;
                let rows_ref = if is_headers { &mut self.edit_headers } else { &mut self.edit_params };
                if let Some(row) = rows_ref.get_mut(idx) {
                    row.enabled = !row.enabled;
                }
                self.save_to_selected_request();
            }
            KeyCode::Enter => {
                // Start editing key
                let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                let idx = kv.selected;
                let rows_ref = if is_headers { &self.edit_headers } else { &self.edit_params };
                if let Some(row) = rows_ref.get(idx) {
                    let kv = if is_headers { &mut self.headers_kv } else { &mut self.params_kv };
                    kv.editing_col = Some(0);
                    kv.edit_buf = row.key.clone();
                }
            }
            _ => {}
        }
    }

    fn handle_auth_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('1') => self.edit_auth = AuthConfig::None,
            KeyCode::Char('2') => {
                if !matches!(self.edit_auth, AuthConfig::BearerToken { .. }) {
                    self.edit_auth = AuthConfig::BearerToken { token: String::new() };
                }
            }
            KeyCode::Char('3') => {
                if !matches!(self.edit_auth, AuthConfig::BasicAuth { .. }) {
                    self.edit_auth = AuthConfig::BasicAuth { username: String::new(), password: String::new() };
                }
            }
            KeyCode::Char('4') => {
                if !matches!(self.edit_auth, AuthConfig::ApiKey { .. }) {
                    self.edit_auth = AuthConfig::ApiKey { header: String::new(), value: String::new() };
                }
            }
            _ => {}
        }
        self.save_to_selected_request();
    }

    // ── Request body key handler ─────────────────────────────────────────────

    fn handle_request_body_key(&mut self, _key: KeyEvent) {
        // Body is handled via insert mode
    }

    // ── Response key handler ─────────────────────────────────────────────────

    fn handle_response_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.response_scroll = self.response_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.response_scroll = self.response_scroll.saturating_sub(1);
            }
            KeyCode::Char('[') => {
                let idx = self.response_tab.index();
                self.response_tab = ResponseTab::from_index((idx + 2) % 3);
            }
            KeyCode::Char(']') => {
                let idx = self.response_tab.index();
                self.response_tab = ResponseTab::from_index((idx + 1) % 3);
            }
            _ => {}
        }
    }

    // ── Insert mode key handler ──────────────────────────────────────────────

    fn handle_insert_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.insert_mode = false;
                self.save_to_selected_request();
            }
            KeyCode::Enter => {
                match self.focus {
                    FocusPanel::RequestUrl => {
                        self.insert_mode = false;
                        self.save_to_selected_request();
                        self.send_request();
                    }
                    FocusPanel::RequestBody => {
                        self.edit_body.push('\n');
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                match self.focus {
                    FocusPanel::RequestUrl => { self.edit_url.pop(); }
                    FocusPanel::RequestBody => { self.edit_body.pop(); }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match self.focus {
                    FocusPanel::RequestUrl => self.edit_url.push(c),
                    FocusPanel::RequestBody => self.edit_body.push(c),
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }

    // ── Modal key handler ────────────────────────────────────────────────────

    fn handle_modal_key(&mut self, key: KeyEvent) -> bool {
        // Extract only the data we need without cloning the full enum
        match &self.modal {
            ActiveModal::NewCollection { .. } | ActiveModal::NewRequest { .. } => {}
            _ => {}
        }
        match std::mem::replace(&mut self.modal, ActiveModal::None) {
            ActiveModal::NewCollection { mut name } => {
                match key.code {
                    KeyCode::Char(c) => name.push(c),
                    KeyCode::Backspace => { name.pop(); }
                    KeyCode::Enter => {
                        if !name.is_empty() {
                            let col = Collection::new(&name);
                            let _ = storage::collections::save(&col);
                            self.collections.push(col);
                            self.rebuild_sidebar();
                        }
                        return false;
                    }
                    KeyCode::Esc => { return false; }
                    _ => {}
                }
                self.modal = ActiveModal::NewCollection { name };
            }
            ActiveModal::NewRequest { mut name, collection_idx: ci } => {
                match key.code {
                    KeyCode::Char(c) => name.push(c),
                    KeyCode::Backspace => { name.pop(); }
                    KeyCode::Enter => {
                        if !name.is_empty() && ci < self.collections.len() {
                            let req = RequestItem::new(&name);
                            self.collections[ci].requests.push(req);
                            self.collections[ci].expanded = true;
                            let col = self.collections[ci].clone();
                            let _ = storage::collections::save(&col);
                            self.rebuild_sidebar();
                        }
                        return false;
                    }
                    KeyCode::Esc => { return false; }
                    _ => {}
                }
                self.modal = ActiveModal::NewRequest { name, collection_idx: ci };
            }
            ActiveModal::ConfirmDelete { target } => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        match target {
                            DeleteTarget::Collection(ci) => {
                                let id = self.collections[ci].id.clone();
                                let _ = storage::collections::delete(&id);
                                self.collections.remove(ci);
                                if self.selected_collection == Some(ci) {
                                    self.selected_collection = None;
                                    self.selected_request = None;
                                }
                                self.rebuild_sidebar();
                            }
                            DeleteTarget::Request { collection_idx, request_idx } => {
                                if collection_idx < self.collections.len() {
                                    let col = &mut self.collections[collection_idx];
                                    if request_idx < col.requests.len() {
                                        col.requests.remove(request_idx);
                                        let col_clone = col.clone();
                                        let _ = storage::collections::save(&col_clone);
                                    }
                                    self.rebuild_sidebar();
                                }
                            }
                        }
                        // self.modal already None from mem::replace
                    }
                    _ => {
                        // Put it back on cancel
                        self.modal = ActiveModal::ConfirmDelete { target };
                    }
                }
            }
            ActiveModal::Environment => {
                self.modal = ActiveModal::Environment; // restore before handler
                self.handle_env_modal_key(key);
            }
            ActiveModal::Help => {
                if !matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
                    self.modal = ActiveModal::Help; // restore if not closing
                }
            }
            ActiveModal::None => {}
        }
        false
    }

    fn handle_env_modal_key(&mut self, key: KeyEvent) {
        if let Some((env_idx, var_idx)) = self.env_editing {
            match key.code {
                KeyCode::Char(c) => self.env_edit_buf.push(c),
                KeyCode::Backspace => { self.env_edit_buf.pop(); }
                KeyCode::Tab | KeyCode::Enter => {
                    // col 0 = key, col 1 = value
                    // We encode as: env_editing = Some((env_idx, var_idx * 2 + col))
                    let col = var_idx % 2;
                    let actual_var = var_idx / 2;
                    if env_idx < self.environments.len() {
                        let env = &mut self.environments[env_idx];
                        if let Some(v) = env.vars.get_mut(actual_var) {
                            if col == 0 { v.key = self.env_edit_buf.clone(); }
                            else { v.value = self.env_edit_buf.clone(); }
                        }
                        let env_clone = env.clone();
                        let _ = storage::environments::save(&env_clone);
                    }
                    if col == 0 {
                        self.env_editing = Some((env_idx, actual_var * 2 + 1));
                        // Prefill value
                        if let Some(env) = self.environments.get(env_idx) {
                            if let Some(v) = env.vars.get(actual_var) {
                                self.env_edit_buf = v.value.clone();
                            }
                        }
                    } else {
                        self.env_editing = None;
                        self.env_edit_buf.clear();
                    }
                }
                KeyCode::Esc => {
                    self.env_editing = None;
                    self.env_edit_buf.clear();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.modal = ActiveModal::None,
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.environments.len();
                if len > 0 && self.env_list_cursor + 1 < len {
                    self.env_list_cursor += 1;
                    self.env_var_cursor = 0;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.env_list_cursor > 0 {
                    self.env_list_cursor -= 1;
                    self.env_var_cursor = 0;
                }
            }
            KeyCode::Char('n') => {
                // New environment
                let name = format!("env-{}", self.environments.len() + 1);
                let env = Environment::new(&name);
                let _ = storage::environments::save(&env);
                self.environments.push(env);
                self.env_list_cursor = self.environments.len() - 1;
            }
            KeyCode::Char('o') => {
                // Add var to current env
                if let Some(env) = self.environments.get_mut(self.env_list_cursor) {
                    use crate::models::environment::EnvVar;
                    env.vars.push(EnvVar::new("", ""));
                    let var_idx = env.vars.len() - 1;
                    self.env_var_cursor = var_idx;
                    self.env_editing = Some((self.env_list_cursor, var_idx * 2));
                    self.env_edit_buf.clear();
                }
            }
            KeyCode::Enter => {
                // Edit selected var
                if let Some(env) = self.environments.get(self.env_list_cursor) {
                    if self.env_var_cursor < env.vars.len() {
                        let v = &env.vars[self.env_var_cursor];
                        self.env_edit_buf = v.key.clone();
                        self.env_editing = Some((self.env_list_cursor, self.env_var_cursor * 2));
                    }
                }
            }
            KeyCode::Char('A') | KeyCode::Char('a') => {
                // Set as active environment
                self.active_env_idx = Some(self.env_list_cursor);
                self.status_msg = Some(format!(
                    "Active env: {}",
                    self.environments.get(self.env_list_cursor).map(|e| e.name.as_str()).unwrap_or("")
                ));
            }
            KeyCode::Char('J') => {
                if let Some(env) = self.environments.get(self.env_list_cursor) {
                    let len = env.vars.len();
                    if len > 0 && self.env_var_cursor + 1 < len {
                        self.env_var_cursor += 1;
                    }
                }
            }
            KeyCode::Char('K') => {
                if self.env_var_cursor > 0 {
                    self.env_var_cursor -= 1;
                }
            }
            _ => {}
        }
    }
}

// Simple base64 encoder (no external dep beyond std)
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { CHARS[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(n & 63) as usize] as char } else { '=' });
    }
    out
}

/// Percent-encode a query parameter key or value per RFC 3986.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            b => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
