use crossterm::style::{Color, ContentStyle};

#[derive(Debug, Default, Clone)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug)]
pub struct TokenStyle {
    pub name: Option<String>,
    pub scope: Vec<String>,
    pub style: Style,
}

#[derive(Debug)]
pub struct Theme {
    pub name: String,
    pub style: Style,
    pub token_style: Vec<TokenStyle>,
}

impl Theme {
    pub fn get_style(&self, scope: &str) -> Option<Style> {
        self.token_style.iter().find_map(|ts| {
            if ts.scope.contains(&scope.to_string()) {
                return Some(ts.style.clone());
            }
            return None;
        })
    }
}

impl Style {
    pub fn convert_to_style(&mut self, fallback_style: &Style) -> ContentStyle {
        let foreground_color = match self.fg {
            Some(col) => col,
            None => fallback_style.fg.unwrap(),
        };

        let background_color = match self.bg {
            Some(col) => col,
            None => fallback_style.bg.unwrap(),
        };

        ContentStyle {
            foreground_color: Some(foreground_color),
            background_color: Some(background_color),
            ..Default::default()
        }
    }
}
