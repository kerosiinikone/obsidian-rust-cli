use std::{fs::File, path::PathBuf, process::exit};

use anyhow::Error;
use chrono::Local;
use clap::Parser;
use cli_core::{config::Config, note::Note, template::TemplArgs};
use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to vault
    #[arg(short, long)]
    pub vault: Option<PathBuf>,

    /// Path to template
    #[arg(short, long)]
    pub template: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let cfg = Config::build(args.vault, args.template).unwrap_or_else(|_| exit(1));

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(cfg).run(terminal);
    ratatui::restore();
    app
}

#[allow(dead_code)]
enum Screen {
    New,
    Show,
}

#[allow(dead_code)]
struct CreatedNote {
    title: String,
    body: String,
}

#[allow(dead_code)]
struct App {
    cfg: Config,
    input: String,
    character_index: usize,
    input_mode: InputMode,
    created_note: Option<CreatedNote>,
    error_msg: Option<String>,
    screen_select: Screen,
    new_note_screen: NewScreen,
    show_note_screen: ShowScreen,
}

#[allow(dead_code)]
struct NewScreen {
    input: String,
    character_index: usize,
    input_mode: InputMode,
    created_note: Option<CreatedNote>,
    error_msg: Option<String>,
    cfg: Config,
}

#[allow(dead_code)]
struct ShowScreen {
    cfg: Config,
}

#[allow(dead_code)]
enum InputMode {
    Normal,
    Editing,
}

impl ShowScreen {
    const fn new(cfg: Config) -> Self {
        Self { cfg }
    }
}

impl NewScreen {
    const fn new(cfg: Config) -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            created_note: None,
            character_index: 0,
            error_msg: None,
            cfg,
        }
    }

    fn exec(&mut self, relay: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match relay.code {
                KeyCode::Char('e') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('q') => {}
                _ => {}
            },
            InputMode::Editing if relay.kind == KeyEventKind::Press => match relay.code {
                KeyCode::Enter if !self.input.trim_ascii().is_empty() => self.submit_idea(),
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            },
            InputMode::Editing => {}
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_idea(&mut self) {
        self.created_note = self.create_new_note().unwrap_or_else(|err| {
            self.error_msg = Some(err.to_string());
            None
        });
        self.input.clear();
        self.reset_cursor();
    }

    fn create_new_note(&mut self) -> Result<Option<CreatedNote>, Error> {
        let mut note_path: PathBuf = self.cfg.vault.clone();
        let formatted = format!("{}", Local::now().format("%Y_%m_%d_%H_%M_%S"));
        let title = format!("Note_{formatted}.md");
        note_path.push(title.clone());

        let handle = File::create(note_path.as_path())?;

        let body = self.cfg.template.render(&TemplArgs {
            body: self.input.clone(),
            date: formatted.clone(),
        })?;

        let mut note = Note::new(&handle, &note_path, Some(body.clone()));
        note.write_file_handle()?;

        Ok(Some(CreatedNote { body, title }))
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ]);
        let [help_area, input_area, info_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "Del".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start typing an idea. ".bold(),
                    "Press ".into(),
                    "Insert".bold(),
                    " to switch to displaying notes.".into(),
                ],
                Style::default(),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop typing, ".into(),
                    "Enter".bold(),
                    " to create a note.".into(),
                ],
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Rgb(126u8, 29u8, 251u8)),
            })
            .block(Block::bordered().title("Idea"));

        frame.render_widget(input, input_area);

        match self.input_mode {
            InputMode::Normal => {}
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16 + 1,
                input_area.y + 1,
            )),
        }

        if let Some(err) = self.error_msg.as_ref() {
            let msg = format!("Error: {err}");
            let text = Text::from(Line::from(msg)).patch_style(style);
            let err_info = Paragraph::new(text);
            frame.render_widget(err_info, info_area);
        } else if let Some(note) = self.created_note.as_ref() {
            let msg = format!("Created a new note: {}", note.title);
            let text = Text::from(Line::from(msg)).patch_style(style);
            let created_info = Paragraph::new(text);
            frame.render_widget(created_info, info_area);
        }
    }
}

impl ShowScreen {
    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Length(1)]);
        let [help_area] = vertical.areas(frame.area());

        let (msg, style) = (
            vec!["Type out the path of the note to display".into()],
            Style::default(),
        );

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);
    }
}

impl App {
    fn new(cfg: Config) -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            created_note: None,
            character_index: 0,
            error_msg: None,
            screen_select: Screen::New,
            new_note_screen: NewScreen::new(cfg.clone()),
            show_note_screen: ShowScreen::new(cfg.clone()),
            cfg,
        }
    }

    // Use dynamic dispatch for diff screens?
    fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| match self.screen_select {
                Screen::New => self.new_note_screen.draw(frame),
                Screen::Show => self.show_note_screen.draw(frame),
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Delete => {
                        return Ok(());
                    }
                    KeyCode::Insert if key.kind == KeyEventKind::Press => {
                        self.screen_select = if let Screen::Show = self.screen_select {
                            Screen::New
                        } else {
                            Screen::Show
                        }
                    }
                    _ => {
                        match self.screen_select {
                            Screen::New => self.new_note_screen.exec(key),
                            // Screen::Show => self.show_note_screen.exec(relay),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
