use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, read},
    style, terminal, ExecutableCommand, QueueableCommand,
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
        println!("the size is {:?}" ,self.cx); 
        self.stdout.execute(MoveTo(0,  self.size.1 - 2))?;
        self.stdout.queue(style::Print("Status Line"))?;
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

        self.stdout.execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    fn handle_event(&mut self, event: event::Event) -> anyhow::Result<Option<Action>> {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(event),
            Mode::Insert => self.handle_insert_mode(event),
        }
    }

    fn handle_normal_mode(&self, event: event::Event) -> anyhow::Result<Option<Action>> {
        match event {
            event::Event::Key(ev) => match ev.code {
                event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                event::KeyCode::Char('h') | event::KeyCode::Left => Ok(Some(Action::MoveLeft)),
                event::KeyCode::Char('j') | event::KeyCode::Down => Ok(Some(Action::MoveDown)),
                event::KeyCode::Char('k') | event::KeyCode::Up => Ok(Some(Action::MoveUp)),
                event::KeyCode::Char('l') | event::KeyCode::Right => Ok(Some(Action::MoveRight)),
                event::KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),

                _ => Ok(None),
            },

            _ => Ok(None),
        }
    }

    fn handle_insert_mode(&mut self, event: event::Event) -> anyhow::Result<Option<Action>> {
        match event {
            event::Event::Key(key) => match key.code {
                event::KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
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
}
