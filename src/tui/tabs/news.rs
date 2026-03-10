// render list
// render content

use crate::utils::cna_model::CNAModel;
use ratatui::widgets::ListState;

pub enum Focused {
    Left,
    Right,
}

pub struct News {
    pub items: Vec<CNAModel>,
    pub state: ListState,
}

impl News {
    pub fn new(items: Vec<CNAModel>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { items, state }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
