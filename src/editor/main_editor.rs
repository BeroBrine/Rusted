use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor::{self, MoveTo},
    event::{self, read, KeyModifiers},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};

use super::action::Action;
use super::mode::Mode;
use crate::{log, Buffer};

pub struct Editor {
    buffer: Buffer,
    stdout: Stdout,
    mode: Mode,
    size: (u16, u16),
    vtop: u16,
    vleft: u16,
    vheight: u16,
    vwidth: u16,
    cx: u16,
    cy: u16,
}

impl Editor {
    pub fn new(file_buffer: Buffer) -> anyhow::Result<Self> {
        let size = terminal::size()?;
        Ok(Editor {
            buffer: file_buffer,
            mode: Mode::Normal,
            vtop: 0,
            vleft: 0,
            cx: 0,
            cy: 0,
            vheight: size.1 - 2,
            vwidth: size.0,
            size,
            stdout: stdout(),
        })
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.draw_viewport()?;
        self.draw_statusline()?;
        self.stdout.queue(MoveTo(self.cx, self.cy))?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn draw_viewport(&mut self) -> anyhow::Result<()> {
        let vwidth = self.vwidth;
        for i in 0..self.vheight {
            let line = self.viewport_line(i);
            let print_line = match line {
                Some(val) => val,
                None => String::new(),
            };

            self.stdout.queue(cursor::MoveTo(0, i as u16))?;
            let format_string = format!("{print_line:<width$}", width = vwidth as usize);
            self.stdout.queue(style::Print(format_string))?;
        }
        Ok(())
    }

    fn viewport_line(&mut self, n: u16) -> Option<String> {
        let buf_line = self.vtop + n;
        self.buffer.get(buf_line as usize)
    }

    fn draw_statusline(&mut self) -> anyhow::Result<()> {
        self.stdout.execute(MoveTo(0, self.size.1 - 2))?;
        let mode = self.get_mode().to_uppercase();
        let pos = format!(" {}:{} ", self.cx, self.cy + self.vtop);
        let file = format!(" {} ", self.buffer.file.as_deref().unwrap_or("No Name"));
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
    fn get_line_length(&self) -> u16 {
        let line_in_buf = self.cy + self.vtop;
        if let Some(val) = self.buffer.get(line_in_buf as usize) {
            return val.len() as u16;
        }
        0
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
            let buf_end = self.buffer.lines.len() as u16;
            if let Some(event) = event {
                match &event {
                    Action::Quit => break,
                    Action::MoveRight => {
                        self.cx = self.cx.saturating_add(1);
                    }
                    Action::MoveLeft => {
                        self.cx = self.cx.saturating_sub(1);
                        if self.cx < self.vleft {
                            self.cx = self.vleft;
                        }
                    }
                    Action::MoveDown => {
                        self.cy = self.cy.saturating_add(1);
                    }
                    Action::MoveUp => {
                        self.cy = self.cy.saturating_sub(1);
                    }
                    Action::PageUp => {
                        if self.vtop > 0 {
                            self.vtop = self.vtop.saturating_sub(self.vheight);
                        }
                        if self.vtop == 0 {
                            self.cy = 0;
                        }
                    }
                    Action::PageDown => {
                        if self.vtop + self.vheight < buf_end {
                            self.vtop += self.vheight;
                        }
                        if self.vtop + self.vheight > buf_end {
                            self.cy = buf_end.saturating_sub(self.vtop);
                        }
                    }
                    Action::EnterMode(mode) => match mode {
                        Mode::Insert => self.mode = Mode::Insert,
                        Mode::Normal => self.mode = Mode::Normal,
                    },
                };
                self.check_bounds(&event, buf_end)?;
            };
        }

        Ok(())
    }

    fn check_bounds(&mut self, action: &Action, buf_end: u16) -> anyhow::Result<()> {
        let line_length = self.get_line_length();
        match action {
            Action::MoveRight => {
                if self.cx >= line_length {
                    self.cx = line_length.saturating_sub(1);
                }

                if self.cx >= self.vwidth {
                    self.cx = self.vwidth;
                }
            }

            Action::MoveUp => {
                if self.cx >= line_length {
                    self.cx = line_length.saturating_sub(1);
                }
                if self.cy == 0 {
                    if self.vtop > 0 {
                        self.vtop = self.vtop.saturating_sub(1);
                    }
                }
            }
            Action::MoveDown => {
                if self.cx >= self.get_line_length() {
                    self.cx = self.get_line_length();
                }
                if self.cy >= self.vheight as u16 {
                    self.cy = self.vheight.saturating_sub(1) as u16;
                    if self.vtop + self.vheight < buf_end {
                        self.vtop += 1;
                    }
                }
                if self.cy + self.vtop >= buf_end + 1{
                    self.cy = buf_end.saturating_sub(self.vtop);
                }
            }
            _ => (),
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
            event::Event::Key(ev) => {
                let code = ev.code;
                let modifier = ev.modifiers;
                match code {
                    event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                    event::KeyCode::Char('h') | event::KeyCode::Left => Ok(Some(Action::MoveLeft)),
                    event::KeyCode::Char('j') | event::KeyCode::Down => Ok(Some(Action::MoveDown)),
                    event::KeyCode::Char('k') | event::KeyCode::Up => Ok(Some(Action::MoveUp)),
                    event::KeyCode::Char('l') | event::KeyCode::Right => {
                        Ok(Some(Action::MoveRight))
                    }
                    event::KeyCode::Char('i') => {
                        self.stdout.execute(cursor::EnableBlinking)?;
                        Ok(Some(Action::EnterMode(Mode::Insert)))
                    }
                    event::KeyCode::Char('f') => {
                        if matches!(modifier, KeyModifiers::CONTROL) {
                            Ok(Some(Action::PageDown))
                        } else {
                            Ok(None)
                        }
                    }
                    event::KeyCode::Char('b') => {
                        if matches!(modifier, KeyModifiers::CONTROL) {
                            Ok(Some(Action::PageUp))
                        } else {
                            Ok(None)
                        }
                    }

                    _ => Ok(None),
                }
            }

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

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.stdout.flush();
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
