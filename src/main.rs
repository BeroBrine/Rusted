use std::io::stdout;

use crossterm::{cursor::MoveTo, event::read, terminal, ExecutableCommand};

fn main() -> anyhow::Result<()> {
    let mut stdout = stdout();
    let cx = 0;
    let cy = 0;

    terminal::enable_raw_mode()?;

    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    stdout.execute(MoveTo(cx, cy))?;
    loop {
        match read()? {
            crossterm::event::Event::Key(event) => match event.code {
                crossterm::event::KeyCode::Char('q') => break,
                _ => (),
            },
            _ => (),
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
