use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::tui::{display::Message, tabs::news::News};

pub enum Focused {
    Left,
    Right,
}

pub enum Mode {
    Normal,
    Insert, // should only work for the search feature
    Visual, // should only work in content to select content
}

#[derive(PartialEq)]
pub enum Tab {
    News,
    Papers,
    Custom,
}

pub struct App {
    pub focused: Focused,
    pub mode: Mode,
    pub tab: Tab,
    pub news_app: News,
    // pub papers_app: Papers,
    // pub custom_app: Custom,
    pub tx: Sender<Message>,
    pub rx: Receiver<Message>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(32);
        Self {
            focused: Focused::Left,
            mode: Mode::Normal,
            tab: Tab::News,
            news_app: News::new(),
            tx,
            rx,
        }
    }
}
