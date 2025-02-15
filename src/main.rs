pub mod editor;

use editor::main_editor::Editor;

fn main() -> anyhow::Result<()> {
    let mut editor = Editor::new()?;
    editor.init_editor()?;

    Ok(())
}
