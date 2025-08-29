use cli_core::config::Config;
use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};

use crate::{new::NewScreen, show::ShowScreen};

enum Screen {
    New,
    Show,
}
pub(crate) struct App {
    cfg: Config,
    screen_select: Screen,
    new_note_screen: NewScreen,
    show_note_screen: ShowScreen,
}

impl App {
    pub(crate) fn new(cfg: Config) -> Self {
        Self {
            screen_select: Screen::New,
            new_note_screen: NewScreen::new(),
            show_note_screen: ShowScreen::new(),
            cfg,
        }
    }

    pub(crate) fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
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
                    _ => match self.screen_select {
                        Screen::New => self.new_note_screen.exec(key, &self.cfg),
                        Screen::Show => self.show_note_screen.exec(key, &self.cfg),
                    },
                }
            }
        }
    }
}
