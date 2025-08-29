use cli_core::config::Config;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
};
use std::{fs::File, io::Read, path::PathBuf};

use crate::input::Input;

pub(crate) struct ShowScreen {
    input: Input,
    vertical_scroll: usize,
    note_content: Option<String>,
    error_msg: Option<String>,
}

impl ShowScreen {
    pub(crate) const fn new() -> Self {
        Self {
            vertical_scroll: 0,
            error_msg: None,
            note_content: None,
            input: Input::new(),
        }
    }

    pub(crate) fn exec(&mut self, relay: KeyEvent, cfg: &Config) {
        if relay.kind == KeyEventKind::Press {
            match relay.code {
                KeyCode::Enter if !self.input.input.trim_ascii().is_empty() => {
                    self.search(cfg).unwrap_or_else(|err| {
                        self.error_msg = Some(err.to_string());
                    })
                }
                KeyCode::Down => {
                    self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                }
                KeyCode::Up => {
                    self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                }
                KeyCode::Char(to_insert) => self.input.enter_char(to_insert),
                KeyCode::Backspace => self.input.delete_char(),
                KeyCode::Left => self.input.move_cursor_left(),
                KeyCode::Right => self.input.move_cursor_right(),
                _ => {}
            }
        }
    }

    pub(crate) fn search(&mut self, cfg: &Config) -> anyhow::Result<()> {
        let abs_path = cfg.get_full_path(&PathBuf::from(&self.input.input))?;
        let mut handle = File::open(abs_path.as_path())?;

        let mut buf = String::new();
        handle.read_to_string(&mut buf)?;

        self.note_content = Some(buf);
        self.input.input.clear();
        self.input.reset_cursor();
        self.error_msg = None;

        Ok(())
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Max(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ]);
        let [help_area, input_area, note_area, info_area] = vertical.areas(frame.area());

        self.render_help(frame, help_area);
        self.render_input(frame, input_area);
        self.render_note(frame, note_area);
        self.render_error(frame, info_area);
    }

    fn render_error(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(err) = self.error_msg.as_ref() {
            let msg = format!("Error: {err}");
            let text = Text::from(Line::from(msg)).patch_style(Style::default());
            let err_info = Paragraph::new(text);
            frame.render_widget(err_info, area);
        }
    }

    fn render_help(&mut self, frame: &mut Frame, area: Rect) {
        let (msg, style) = (
            vec![
                "Type out the path of the note to display. ".not_bold(),
                "Use ".not_bold(),
                "Arrows ".bold(),
                "to scroll the note.".not_bold(),
            ],
            Style::default(),
        );
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, area);
    }

    fn render_input(&mut self, frame: &mut Frame, area: Rect) {
        let input = Paragraph::new(self.input.input.as_str())
            .style(Style::default().fg(Color::Rgb(126u8, 29u8, 251u8)))
            .block(Block::bordered().title("Path (inside the vault)"));
        frame.render_widget(input, area);

        frame.set_cursor_position(Position::new(
            area.x + self.input.character_index as u16 + 1,
            area.y + 1,
        ));
    }

    fn render_note(&mut self, frame: &mut Frame, area: Rect) {
        let note = Paragraph::new(if let Some(note_content) = &self.note_content {
            note_content
        } else {
            ""
        })
        .scroll((self.vertical_scroll as u16, 0));
        frame.render_widget(note, area);
    }
}
