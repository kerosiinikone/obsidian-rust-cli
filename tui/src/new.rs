use anyhow::Error;
use chrono::Local;
use cli_core::{config::Config, note::Note, template::TemplArgs};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
};
use std::{fs::File, path::PathBuf};

use crate::input::{Input, InputMode};

#[allow(dead_code)]
struct CreatedNote {
    title: String,
    body: String,
}

pub(crate) struct NewScreen {
    input: Input,
    input_mode: InputMode,
    created_note: Option<CreatedNote>,
    error_msg: Option<String>,
}

impl NewScreen {
    pub(crate) const fn new() -> Self {
        Self {
            input: Input::new(),
            input_mode: InputMode::Normal,
            created_note: None,
            error_msg: None,
        }
    }

    pub(crate) fn exec(&mut self, relay: KeyEvent, cfg: &Config) {
        match self.input_mode {
            InputMode::Normal => match relay.code {
                KeyCode::Char('e') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('q') => {}
                _ => {}
            },
            InputMode::Editing if relay.kind == KeyEventKind::Press => match relay.code {
                KeyCode::Enter if !self.input.input.trim_ascii().is_empty() => {
                    self.submit_idea(cfg)
                }
                KeyCode::Char(to_insert) => self.input.enter_char(to_insert),
                KeyCode::Backspace => self.input.delete_char(),
                KeyCode::Left => self.input.move_cursor_left(),
                KeyCode::Right => self.input.move_cursor_right(),
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            },
            InputMode::Editing => {}
        }
    }

    pub(crate) fn submit_idea(&mut self, cfg: &Config) {
        self.created_note = self.create_new_note(cfg).unwrap_or_else(|err| {
            self.error_msg = Some(err.to_string());
            None
        });
        self.input.input.clear();
        self.input.reset_cursor();
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ]);
        let [help_area, input_area, info_area] = vertical.areas(frame.area());

        self.render_help(frame, help_area);
        self.render_input(frame, input_area);
        self.render_info_error(frame, info_area);
    }

    fn create_new_note(&mut self, cfg: &Config) -> Result<Option<CreatedNote>, Error> {
        let mut note_path: PathBuf = cfg.vault.clone();
        let formatted = format!("{}", Local::now().format("%Y_%m_%d_%H_%M_%S"));
        let title = format!("Note_{formatted}.md");
        note_path.push(title.clone());

        let handle = File::create(note_path.as_path())?;

        let body = cfg.template.render(&TemplArgs {
            body: self.input.input.clone(),
            date: formatted.clone(),
        })?;

        let mut note = Note::new(&handle, &note_path, Some(body.clone()));
        note.write_file_handle()?;

        Ok(Some(CreatedNote { body, title }))
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".not_bold(),
                    "Del".bold(),
                    " to exit, ".not_bold(),
                    "e".bold(),
                    " to start typing an idea. ".bold(),
                    "Press ".not_bold(),
                    "Insert".bold(),
                    " to switch to displaying notes.".not_bold(),
                ],
                Style::default(),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".not_bold(),
                    "Esc".bold(),
                    " to stop typing, ".not_bold(),
                    "Enter".bold(),
                    " to create a note.".not_bold(),
                ],
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let input = Paragraph::new(self.input.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Rgb(126u8, 29u8, 251u8)),
            })
            .block(Block::bordered().title("Idea"));

        frame.render_widget(input, area);

        match self.input_mode {
            InputMode::Normal => {}
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                area.x + self.input.character_index as u16 + 1,
                area.y + 1,
            )),
        }
    }

    fn render_info_error(&self, frame: &mut Frame, area: Rect) {
        let style = Style::default();
        if let Some(err) = self.error_msg.as_ref() {
            let msg = format!("Error: {err}");
            let text = Text::from(Line::from(msg)).patch_style(style);
            let err_info = Paragraph::new(text);
            frame.render_widget(err_info, area);
        } else if let Some(note) = self.created_note.as_ref() {
            let msg = format!("Created a new note: {}", note.title);
            let text = Text::from(Line::from(msg)).patch_style(style);
            let created_info = Paragraph::new(text);
            frame.render_widget(created_info, area);
        }
    }
}
