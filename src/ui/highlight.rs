use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

pub struct Highlighter {
    ps: SyntaxSet,
    ts: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_json<'a>(&self, text: &str) -> Vec<Line<'static>> {
        let syntax = self
            .ps
            .find_syntax_by_extension("json")
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());
        let theme = &self.ts.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);

        let mut lines: Vec<Line<'static>> = Vec::new();
        for line in LinesWithEndings::from(text) {
            let regions = h.highlight_line(line, &self.ps).unwrap_or_default();
            let spans: Vec<Span<'static>> = regions
                .iter()
                .map(|(style, text)| {
                    let fg = syntect_to_ratatui_color(style.foreground);
                    let mut rstyle = Style::default().fg(fg);
                    if style.font_style.contains(FontStyle::BOLD) {
                        rstyle = rstyle.add_modifier(Modifier::BOLD);
                    }
                    if style.font_style.contains(FontStyle::ITALIC) {
                        rstyle = rstyle.add_modifier(Modifier::ITALIC);
                    }
                    // Strip trailing newline for display
                    let s = text.trim_end_matches('\n').to_string();
                    Span::styled(s, rstyle)
                })
                .collect();
            lines.push(Line::from(spans));
        }
        lines
    }
}

fn syntect_to_ratatui_color(c: syntect::highlighting::Color) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}
