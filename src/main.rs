mod buffer;
mod editor;
#[macro_use]
mod logger;

use buffer::_buffer::Buffer;
use editor::main_editor::Editor;

fn main() -> anyhow::Result<()> {
    let arg = std::env::args().nth(1);
    let file_buffer = Buffer::from_file(arg);
    log!("he");
    let mut editor = Editor::new(file_buffer)?;
    editor.init_editor()?;

    Ok(())
}
