use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor::{self, MoveTo},
    event::{self, read},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};

use super::action::Action;
use super::mode::Mode;

pub struct Editor {
    stdout: Stdout,
    mode: Mode,
    size: (u16, u16),
    cx: u16,
    cy: u16,
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.stdout.flush();
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

impl Editor {
    fn default() -> anyhow::Result<Editor> {
        Ok(Editor {
            mode: Mode::Normal,
            cx: 0,
            size: terminal::size()?,
            stdout: stdout(),
            cy: 0,
        })
    }

    pub fn new() -> anyhow::Result<Editor> {
        Self::default()
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.draw_statusline()?;
        self.stdout.queue(MoveTo(self.cx, self.cy))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_statusline(&mut self) -> anyhow::Result<()> {
        self.stdout.execute(MoveTo(0, self.size.1 - 2))?;
        let mode = self.get_mode().to_uppercase();
        let pos = format!(" {}:{} ", self.cx, self.cy);
        let file = format!(" src/main.rs");
        let file_width = self.size.0 as usize - mode.len() - pos.len() - 2; // -2 for the
                                                                            // seperators in mode
        self.stdout
            .queue(style::PrintStyledContent("█".with(Color::Rgb {
                r: 184,
                g: 144,
                b: 243,
            })))?;
        self.stdout.queue(style::PrintStyledContent(
            mode.with(Color::Rgb { r: 0, g: 0, b: 0 }).on(Color::Rgb {
                r: 184,
                g: 144,
                b: 243,
            }),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            "█"
                .with(Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                })
                .on(Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            format!("{:<width$}", file, width = file_width as usize)
                .with(Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                })
                .on(Color::Rgb {
                    r: 128,
                    g: 128,
                    b: 128,
                }),
        ))?;

        self.stdout.queue(style::PrintStyledContent(
            pos.with(Color::Rgb { r: 0, g: 0, b: 0 }).on(Color::Rgb {
                r: 184,
                g: 144,
                b: 243,
            }),
        ))?;
        Ok(())
    }

    pub fn init_editor(&mut self) -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        self.stdout.execute(MoveTo(self.cx, self.cy))?;

        loop {
            self.draw()?;
            let event = self.handle_event(read()?)?;
            if let Some(event) = event {
                match event {
                    Action::Quit => break,
                    Action::MoveRight => self.cx = self.cx.saturating_add(1),
                    Action::MoveLeft => {
                        self.cx = self.cx.saturating_sub(1);
                    }
                    Action::MoveDown => {
                        self.cy = self.cy.saturating_add(1);
                    }
                    Action::MoveUp => {
                        self.cy = self.cy.saturating_sub(1);
                    }
                    Action::EnterMode(mode) => match mode {
                        Mode::Insert => self.mode = Mode::Insert,
                        Mode::Normal => self.mode = Mode::Normal,
                    },
                };
            };
        }

        Ok(())
    }

    fn handle_event(&mut self, event: event::Event) -> anyhow::Result<Option<Action>> {
        if matches!(event, event::Event::Resize(_, _)) {
            self.size = terminal::size()?;
        }
        match self.mode {
            Mode::Normal => self.handle_normal_mode(event),
            Mode::Insert => self.handle_insert_mode(event),
        }
    }

    fn handle_normal_mode(&mut self, event: event::Event) -> anyhow::Result<Option<Action>> {
        match event {
            event::Event::Key(ev) => match ev.code {
                event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                event::KeyCode::Char('h') | event::KeyCode::Left => Ok(Some(Action::MoveLeft)),
                event::KeyCode::Char('j') | event::KeyCode::Down => Ok(Some(Action::MoveDown)),
                event::KeyCode::Char('k') | event::KeyCode::Up => Ok(Some(Action::MoveUp)),
                event::KeyCode::Char('l') | event::KeyCode::Right => Ok(Some(Action::MoveRight)),
                event::KeyCode::Char('i') => {
                    self.stdout.execute(cursor::EnableBlinking)?;
                    Ok(Some(Action::EnterMode(Mode::Insert)))
                }

                _ => Ok(None),
            },

            _ => Ok(None),
        }
    }

    fn handle_insert_mode(&mut self, event: event::Event) -> anyhow::Result<Option<Action>> {
        match event {
            event::Event::Key(key) => match key.code {
                event::KeyCode::Esc => {
                    self.stdout.execute(cursor::DisableBlinking)?;
                    Ok(Some(Action::EnterMode(Mode::Normal)))
                }
                event::KeyCode::Char(c) => {
                    self.stdout.queue(style::Print(c))?;
                    self.cx += 1;
                    Ok(None)
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn get_mode(&self) -> String {
        match self.mode {
            Mode::Insert => String::from("Insert"),
            Mode::Normal => String::from("Normal"),
        }
    }
}
