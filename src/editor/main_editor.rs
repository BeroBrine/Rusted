use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor::{self, MoveTo, SetCursorStyle},
    event::{self, read, KeyModifiers},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};

use super::action::Action;
use super::mode::Mode;
use crate::{log, Buffer};

#[derive(Debug)]
pub struct InsertChanges {
    pub index: (u16, u16),
    pub line_no: u16,
}

pub struct Editor {
    buffer: Buffer,
    stdout: Stdout,
    mode: Mode,
    size: (u16, u16),
    vtop: u16,
    vleft: u16,
    vheight: u16,
    vwidth: u16,
    cursor_style: SetCursorStyle,
    cx: u16,
    cy: u16,
    waiting_cmd: Option<char>,
    undo_actions_list: Vec<Action>,
    undo_cursor_pos: (u16, u16), // insert mode enter and exit cursor pos
    undo_buffer_list: Vec<(String, u16)>, // string and the index
}

impl Editor {
    pub fn new(file_buffer: Buffer) -> anyhow::Result<Self> {
        let size = terminal::size()?;
        Ok(Editor {
            buffer: file_buffer,
            mode: Mode::Normal,
            vtop: 0,
            cursor_style: SetCursorStyle::DefaultUserShape,
            vleft: 0,
            cx: 0,
            cy: 0,
            vheight: size.1 - 2,
            vwidth: size.0,
            undo_cursor_pos: (0, 0),
            size,
            undo_actions_list: vec![],
            undo_buffer_list: vec![],
            waiting_cmd: None,
            stdout: stdout(),
        })
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.stdout.execute(self.cursor_style)?;
        self.draw_viewport()?;
        self.draw_statusline()?;
        self.stdout.queue(MoveTo(self.cx, self.cy))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_viewport(&mut self) -> anyhow::Result<()> {
        let vwidth = self.vwidth;
        // let buf_end = self.buffer.lines.len().saturating_sub(1) as u16;
        for i in 0..self.vheight {
            let line = self.viewport_line(i);
            let print_line = match line {
                Some(val) => val,
                None => String::new(),
            };
            // let line_no = self.vtop + i + 1;
            // self.stdout.queue(cursor::MoveTo(0, i))?;
            // if line_no <= buf_end + 1 {
            //     self.stdout.queue(style::Print(format!("{line_no} ")))?;
            // }
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
    pub fn get_line_length(&self) -> u16 {
        let line_in_buf = self.get_buf_line();
        if let Some(val) = self.buffer.get(line_in_buf as usize) {
            return val.len() as u16;
        }
        0
    }

    pub fn get_buf_line(&self) -> u16 {
        self.vtop + self.cy
    }

    pub fn init_editor(&mut self) -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        self.stdout.execute(self.cursor_style)?;
        self.stdout.execute(MoveTo(self.cx, self.cy))?;

        loop {
            self.draw()?;
            let event = self.handle_event(read()?)?;
            if let Some(ev) = &event {
                if matches!(ev, Action::Quit) {
                    break;
                }
            }
            let buf_end = self.buffer.lines.len() as u16;
            self.handle_action(&event);
            self.check_bounds(&event, buf_end)?;
        }

        Ok(())
    }

    fn check_bounds(&mut self, action: &Option<Action>, buf_end: u16) -> anyhow::Result<()> {
        let line_length = self.get_line_length();
        if let Some(action) = action {
            match action {
                Action::MoveRight => {
                    if self.cx >= line_length {
                        self.cx = line_length.saturating_sub(1);
                    }

                    if self.cx >= self.vwidth {
                        self.cx = self.vwidth;
                    }
                }
                Action::InsertLineBelowCursor => {
                    if self.cx > line_length {
                        self.cx = line_length;
                    }
                }
                Action::MoveLeft => {
                    if self.cx <= self.vleft {
                        self.cx = self.vleft;
                    }
                }

                Action::MoveUp => {
                    if self.cx > line_length {
                        self.cx = line_length;
                    }
                }

                Action::MoveDown => {
                    if self.cx >= line_length {
                        self.cx = line_length.saturating_sub(1);
                    }
                    if self.cy >= self.vheight as u16 {
                        self.cy = self.vheight.saturating_sub(1) as u16;
                        if self.vtop + self.vheight < buf_end {
                            self.vtop += 1;
                        }
                    }
                    if self.cy + self.vtop >= buf_end {
                        self.cy = buf_end.saturating_sub(self.vtop);
                        self.cy = self.cy.saturating_sub(1);
                    }
                }
                Action::PageDown => {
                    if self.cy + self.vtop >= buf_end {
                        self.cy = buf_end.saturating_sub(self.vtop + 1);
                    }
                }
                Action::PageUp => {
                    if self.vtop == 0 {
                        self.cy = 0;
                    }
                }
                _ => (),
            }
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
        if let Some(char) = self.waiting_cmd {
            self.waiting_cmd = None;
            return self.handle_wait_event(char, event);
        }
        match event {
            event::Event::Key(ev) => {
                let code = ev.code;
                let modifier = ev.modifiers;
                match code {
                    event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
                    event::KeyCode::Char('h') | event::KeyCode::Left => Ok(Some(Action::MoveLeft)),
                    event::KeyCode::Char('j') | event::KeyCode::Down => Ok(Some(Action::MoveDown)),
                    event::KeyCode::Char('k') | event::KeyCode::Up => Ok(Some(Action::MoveUp)),
                    event::KeyCode::Char('u') => Ok(Some(Action::Undo)),
                    event::KeyCode::Char('o') => Ok(Some(Action::InsertLineBelowCursor)),
                    event::KeyCode::Char('G') => Ok(Some(Action::GoToEndOfBuffer)),

                    event::KeyCode::Char('l') | event::KeyCode::Right => {
                        Ok(Some(Action::MoveRight))
                    }
                    event::KeyCode::Char('i') => {
                        return self.enter_insert_mode();
                    }
                    event::KeyCode::Char('$') => Ok(Some(Action::MoveToEndOfLine)),
                    event::KeyCode::Char('0') => Ok(Some(Action::MoveToBeginningOfLine)),
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

                    event::KeyCode::Char('d') => Ok(Some(Action::EnterWaitingMode('d'))),
                    event::KeyCode::Char('x') => Ok(Some(Action::DeleteCharCursorPos)),
                    event::KeyCode::Char('z') => Ok(Some(Action::EnterWaitingMode('z'))),
                    event::KeyCode::Char('g') => Ok(Some(Action::EnterWaitingMode('g'))),

                    _ => Ok(None),
                }
            }

            _ => Ok(None),
        }
    }

    fn enter_insert_mode(&mut self) -> anyhow::Result<Option<Action>> {
        self.cursor_style = SetCursorStyle::BlinkingBar;
        log!("entered insert mode when cx was :{} \n", self.cx);
        self.undo_cursor_pos.0 = self.cx;
        self.mode = Mode::Insert;
        Ok(Some(Action::EnterMode(Mode::Insert)))
    }

    fn enter_normal_mode(&mut self) -> anyhow::Result<Option<Action>> {
        self.cursor_style = SetCursorStyle::DefaultUserShape;
        self.undo_cursor_pos.1 = if self.cx == 1 {
            self.cx
        } else {
            self.cx.saturating_sub(1)
        };

        log!("the tuple is now {:?} \n", self.undo_cursor_pos);
        if self.undo_cursor_pos.0 != self.undo_cursor_pos.1 {
            let insert_changes = InsertChanges {
                index: self.undo_cursor_pos,
                line_no: self.get_buf_line(),
            };
            self.undo_actions_list
                .push(Action::UndoInsertChanges(insert_changes));
            log!(
                "the undo aciton list is now {:?} \n",
                self.undo_actions_list
            );
        }
        self.mode = Mode::Normal;
        Ok(Some(Action::EnterMode(Mode::Normal)))
    }

    fn handle_insert_mode(&mut self, event: event::Event) -> anyhow::Result<Option<Action>> {
        match event {
            event::Event::Key(key) => match key.code {
                event::KeyCode::Esc => self.enter_normal_mode(),
                event::KeyCode::Char(c) => Ok(Some(Action::InsertCharCursorPos(c))),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn handle_wait_event(&mut self, c: char, ev: event::Event) -> anyhow::Result<Option<Action>> {
        match c {
            'd' => match ev {
                event::Event::Key(key) => match key.code {
                    event::KeyCode::Char('d') => Ok(Some(Action::DeleteFullLine)),
                    _ => Ok(None),
                },
                _ => Ok(None),
            },
            'z' => match ev {
                event::Event::Key(key) => match key.code {
                    event::KeyCode::Char('z') => Ok(Some(Action::CenterLineToViewport)),
                    _ => Ok(None),
                },

                _ => Ok(None),
            },
            'g' => match ev {
                event::Event::Key(key) => match key.code {
                    event::KeyCode::Char('g') => Ok(Some(Action::GoToStartOfBuffer)),
                    _ => Ok(None),
                },
                _ => Ok(None),
            },

            _ => Ok(None),
        }
    }
    pub fn handle_action(&mut self, event: &Option<Action>) {
        let buf_end = self.buffer.lines.len() as u16;
        let line_length = self.get_line_length();
        let line_no = self.get_buf_line();
        if let Some(event) = event {
            match event {
                Action::Quit => {}
                Action::InsertLineBelowCursor => {
                    let idx = self.vtop + self.cy + 1;
                    log!("the idx is {} \n", idx);
                    self.buffer.insert_line(idx);
                    self.cy += 1;
                    self.cursor_style = SetCursorStyle::BlinkingBar;
                    let _ = self.enter_insert_mode();
                }
                Action::GoToEndOfBuffer => {
                    self.vtop = buf_end.saturating_sub(self.vheight);
                    self.cy = buf_end.saturating_sub(self.vtop);
                    self.cy = self.cy.saturating_sub(1);
                }
                Action::GoToStartOfBuffer => {
                    if self.vtop > 0 {
                        self.vtop = 0;
                    }
                    self.cy = 0;
                }
                Action::Undo => {
                    self.handle_undo_event();
                }
                Action::MoveRight => {
                    self.cx = self.cx.saturating_add(1);
                }
                Action::MoveLeft => {
                    self.cx = self.cx.saturating_sub(1);
                }
                Action::MoveDown => {
                    self.cy = self.cy.saturating_add(1);
                }
                Action::MoveUp => {
                    if self.cy == 0 {
                        if self.vtop > 0 {
                            self.vtop = self.vtop.saturating_sub(1);
                        }
                    }
                    self.cy = self.cy.saturating_sub(1);
                }
                Action::MoveToEndOfLine => {
                    self.cx = line_length - 1;
                }
                Action::InsertCharCursorPos(c) => {
                    // let idx = self.cx.saturating_sub(self.vleft + 1);
                    self.buffer.insert_char(self.cx, line_no, *c);
                    self.cx += 1;
                }
                Action::EnterWaitingMode(char) => {
                    self.waiting_cmd = Some(*char);
                }
                Action::DeleteFullLine => {
                    self.undo_actions_list.push(Action::DeleteFullLine);
                    log!("deleting line at {} \n", line_no);
                    let line = self.buffer.delete_line(line_no);
                    self.undo_buffer_list.push((line, line_no));
                }
                Action::DeleteCharCursorPos => {
                    let line_no = self.get_buf_line();
                    if self.cx < line_length {
                        self.buffer.delete_char(self.cx, line_no);
                    }
                }
                Action::MoveToBeginningOfLine => {
                    self.cx = 0;
                }
                Action::PageUp => {
                    if self.vtop > 0 {
                        self.vtop = self.vtop.saturating_sub(self.vheight);
                    }
                    if self.vtop == 0 {
                        self.cy = 1;
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
                Action::CenterLineToViewport => {
                    let index = self.get_buf_line();
                    self.vtop = index.saturating_sub(self.vheight / 2);
                    self.cy = index.saturating_sub(self.vtop);
                }
                Action::EnterMode(mode) => match mode {
                    Mode::Insert => {
                        self.cursor_style = SetCursorStyle::BlinkingBar;
                        self.mode = Mode::Insert;
                    }
                    Mode::Normal => {
                        self.cursor_style = SetCursorStyle::DefaultUserShape;
                        self.mode = Mode::Normal;
                    }
                },
                _ => (),
            };
        }
    }

    fn get_mode(&mut self) -> String {
        match self.mode {
            Mode::Insert => String::from("Insert"),
            Mode::Normal => String::from("Normal"),
        }
    }

    fn handle_undo_event(&mut self) {
        let last_cmd = self.undo_actions_list.pop();
        if let Some(action) = last_cmd {
            match action {
                Action::DeleteFullLine => {
                    let tuple = self.undo_buffer_list.pop();
                    if let Some((deleted_string, index)) = tuple {
                        // idk how i thought this out but this works.
                        if self.vtop <= index && index <= self.vtop + self.vheight - 1 {
                            // inside the viewport
                            self.cy = index.saturating_sub(self.vtop);
                            self.buffer.restore_line(deleted_string, index);
                        } else {
                            // outside the viewport
                            self.buffer.restore_line(deleted_string, index);
                            self.vtop = index.saturating_sub(self.vheight / 2);
                            self.cy = index.saturating_sub(self.vtop);
                        }
                    }
                }
                Action::UndoInsertChanges(insert_changes) => {
                    let index = insert_changes.line_no;

                    if self.vtop <= index && index <= self.vtop + self.vheight - 1 {
                        self.cy = index.saturating_sub(self.vtop);
                        self.cx = insert_changes.index.0;
                    } else {
                        // outside the viewport
                        self.vtop = index.saturating_sub(self.vheight / 2);
                        self.cy = index.saturating_sub(self.vtop);
                        self.cx = insert_changes.index.0;
                    }
                    self.buffer.remove_insert_changes(insert_changes);
                }

                _ => (),
            }
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
