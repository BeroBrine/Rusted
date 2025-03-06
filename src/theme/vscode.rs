use crate::log;

use crossterm::style::Color;
use serde::Deserialize;
use serde_json::{Map, Value};

use super::_theme::{Style, Theme, TokenStyle};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum VsCodeScope {
    String(String),
    Vec(Vec<String>),
}

impl From<VsCodeScope> for Vec<String> {
    fn from(value: VsCodeScope) -> Self {
        match value {
            VsCodeScope::String(s) => vec![translate_scope(s)],
            VsCodeScope::Vec(v) => v.into_iter().map(translate_scope).collect(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct VsCodeTokenColor {
    name: Option<String>,
    scope: VsCodeScope,
    settings: Map<String, Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VsCodeTheme {
    name: Option<String>,
    #[serde(rename = "type")]
    typ: Option<String>,
    colors: Map<String, Value>,
    token_colors: Vec<VsCodeTokenColor>,
}

fn parse_rgb(hexcode: &str) -> anyhow::Result<Color> {
    if !hexcode.starts_with('#') {
        anyhow::bail!("not a valid hex code");
    }

    let r = &hexcode[1..=2];
    let g = &hexcode[3..=4];
    let b = &hexcode[5..=6];

    let r = u8::from_str_radix(r, 16).unwrap();
    let g = u8::from_str_radix(g, 16).unwrap();
    let b = u8::from_str_radix(b, 16).unwrap();

    log!("the rgb value is {:?} {:?} {:?} \n", r, g, b);

    Ok(Color::Rgb { r, g, b })
}

impl From<VsCodeTokenColor> for TokenStyle {
    fn from(value: VsCodeTokenColor) -> Self {
        let mut style = Style::default();

        if let Some(fg) = value.settings.get("foreground") {
            style.fg = Some(parse_rgb(fg.as_str().unwrap()).unwrap());
        }

        if let Some(bg) = value.settings.get("background") {
            style.bg = Some(parse_rgb(bg.as_str().unwrap()).unwrap());
        }

        if let Some(font_style) = value.settings.get("fontStyle") {
            style.bold = font_style.as_str().unwrap().contains("bold");
            style.italic = font_style.as_str().unwrap().contains("italic");
        }

        Self {
            name: value.name,
            scope: value.scope.into(),
            style,
        }
    }
}

fn translate_scope(vscode_scope: String) -> String {
    if vscode_scope == "meta.function-call.constructor" {
        return "constructor".to_string();
    }
    if vscode_scope == "meta.annotation.rust" {
        return "attribute".to_string();
    }
    return vscode_scope.to_string();
}

pub fn parse_theme(file: &str) -> anyhow::Result<Theme> {
    log!("the file is {file}");
    let open_file = std::fs::read_to_string(file)?;
    let vscode_theme_json: VsCodeTheme = match serde_json::from_str(&open_file) {
        Ok(v) => v,
        Err(e) => panic!("error {:#?}", e),
    };

    let mut token_style: Vec<TokenStyle> = Vec::new();
    for token_color in vscode_theme_json.token_colors {
        token_style.push(token_color.into());
    }

    Ok(Theme {
        name: vscode_theme_json.name.unwrap_or_default(),
        style: Style {
            fg: Some(
                parse_rgb(
                    vscode_theme_json
                        .colors
                        .get("editor.foreground")
                        .expect("not found")
                        .as_str()
                        .expect("already a string"),
                )
                .unwrap(),
            ),
            bg: Some(
                parse_rgb(
                    vscode_theme_json
                        .colors
                        .get("editor.background")
                        .expect("not found")
                        .as_str()
                        .expect("already a string"),
                )
                .unwrap(),
            ),
            bold: false,
            italic: false,
        },
        token_style,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test() {
        log!("{:#?}", parse_theme("./frappe.json"));
    }
}
