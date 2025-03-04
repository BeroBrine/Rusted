mod buffer;
mod editor;
mod logger;
mod theme;

use buffer::_buffer::Buffer;
use editor::main_editor::Editor;
use theme::vscode::parse_theme;

fn main() -> anyhow::Result<()> {
    let arg = std::env::args().nth(1);
    let file_buffer = Buffer::from_file(arg);
    let theme = parse_theme("./latte.json")?;
    let mut editor = Editor::new(theme, file_buffer)?;
    editor.init_editor()?;

    Ok(())
}
