use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

struct ListItem {
    pub text: String,
    pub style: Style,
}

impl ListItem {
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }
}

impl Widget for ListItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.text)
            .style(self.style)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL))
            .render(area, buf);
    }
}

pub struct Sidebar {
    pub titles: Vec<String>,
    pub state: ListState,
    pub focused: bool,
}

impl Widget for &mut Sidebar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let builder = ListBuilder::new(|context| {
            let mut item = ListItem::new(self.titles[context.index].clone());
            if context.is_selected {
                item.style = Style::default().fg(Color::Yellow);
            };
            let main_axis_size = 5;
            (item, main_axis_size)
        });
        let item_count = self.titles.len();
        let list = ListView::new(builder, item_count).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if self.focused {
                    Color::Yellow
                } else {
                    Color::Reset
                })),
        );
        let state = &mut self.state;
        list.render(area, buf, state);
    }
}
