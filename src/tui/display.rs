use std::{fs, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{self, Event, KeyCode, KeyModifiers},
    },
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use ratatui_textarea::{Key, TextArea};

use crate::{
    database::sqlite::db, services::cna::NewsCategoryCNA, tui::{
        app::{App, Focused, Mode, Tab},
        tabs::news::News,
    }, utils::fuzzy::fuzzy_match
};

fn update_news_category(app: &mut App, next: bool) {
    if next {
        app.news_app.category.next();
    } else {
        app.news_app.category.previous();
    }
    app.news_app.items = app
        .tokio_runtime
        .block_on(app.news_app.fetch_news(app.news_app.category.get_current()));
    app.news_app.reload_sidebar();
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    // setup db
    let db = db::new();
    // setup app
    let mut app = App::new();
    // setup news_app
    app.news_app.items = app
        .tokio_runtime
        .block_on(app.news_app.fetch_news(app.news_app.category.get_current()));
    let mut items_index = Vec::new();
    for i in 0..app.news_app.items.len() {
        items_index.push(i);
    }
    app.news_app.display_items = items_index;
    app.news_app.reload_sidebar();
    // setup search area
    let mut search_area = TextArea::default();
    search_area.set_block(Block::default().borders(Borders::ALL));
    loop {
        terminal.draw(|frame| {
            render(frame, &mut app, &search_area);
        })?;
        if crossterm::event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.code.is_esc() {
                    app.mode = Mode::Normal
                }
                match app.mode {
                    Mode::Insert => {
                        // disables newline by blocking control and enter
                        match key.modifiers {
                            KeyModifiers::CONTROL => {
                                continue;
                            }
                            _ => {}
                        }
                        match key.code {
                            KeyCode::Enter => {
                                continue;
                            }
                            // prevent reloading when search area is empty
                            KeyCode::Backspace => {
                                if search_area.is_empty() {
                                    continue;
                                }
                            }
                            _ => {}
                        }
                        search_area.input(key);
                        let mut results = fuzzy_match(
                            search_area.lines().join("").to_string(),
                            app.news_app.items.clone(),
                        );
                        results.sort_by_key(|(s, _, _)| *s);
                        results.reverse();
                        app.news_app.display_items =
                            results.iter().map(|(_, _, i)| i.clone()).collect();
                        if search_area.is_empty() {
                            app.news_app.reset_display_items();
                        }
                        app.news_app.update_state();
                    }
                    Mode::Normal => {
                        // use tab to cycle categories
                        match key.code {
                            KeyCode::Tab => match app.tab {
                                Tab::News => update_news_category(&mut app, true),
                                Tab::Papers => {}
                                Tab::Custom => {}
                            },
                            KeyCode::BackTab => match app.tab {
                                Tab::News => update_news_category(&mut app, false),
                                Tab::Papers => {}
                                Tab::Custom => {}
                            },
                            KeyCode::Char('1') => app.tab = Tab::News,
                            KeyCode::Char('2') => app.tab = Tab::Papers,
                            KeyCode::Char('3') => app.tab = Tab::Custom,
                            KeyCode::Char('4') => fs::write(
                                "output.txt",
                                app.news_app.items[app.news_app.sidebar.state.selected.unwrap()]
                                    .content
                                    .clone()
                                    .unwrap()
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<String>(),
                            )
                            .unwrap(),
                            KeyCode::Char('5') => db.save_news(app.news_app.items.clone()),
                            KeyCode::Char('i') => app.mode = Mode::Insert,
                            KeyCode::Char('v') => app.mode = Mode::Visual,
                            KeyCode::Char('h') => app.focused = Focused::Left,
                            KeyCode::Char('l') => app.focused = Focused::Right,
                            KeyCode::Char('j') => match app.focused {
                                Focused::Left => app.news_app.next(),
                                Focused::Right => app.news_app.scroll_down(),
                            },
                            KeyCode::Char('k') => match app.focused {
                                Focused::Left => app.news_app.previous(),
                                Focused::Right => app.news_app.scroll_up(),
                            },
                            KeyCode::Char('q') => break Ok(()),
                            _ => {}
                        }
                    }
                    Mode::Visual => {}
                }
            }
        }
    }
}

fn count_wrapped_lines(text: &str, width: u16) -> u16 {
    if width == 0 {
        return 1;
    }
    let width = width as usize;
    let mut lines = 1u16;
    let mut current_len = 0usize;
    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        if current_len == 0 {
            current_len = word_len;
        } else if current_len + 1 + word_len > width {
            lines += 1;
            current_len = word_len;
        } else {
            current_len += 1 + word_len;
        }
    }
    lines.max(1)
}

fn testing_block(frame: &mut Frame, word: &str, selected: bool, layout: Rect) {
    frame.render_widget(
        Paragraph::new(word)
            .bg(if selected { Color::Green } else { Color::Reset })
            .block(Block::new().borders(Borders::ALL)),
        layout,
    );
}

fn render(frame: &mut Frame, app: &mut App, search_area: &TextArea) {
    let base_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(frame.area());
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(base_layout[1]);
    let tab_layout = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .flex(Flex::Center)
    .spacing(2)
    .split(base_layout[0]);
    let sidebar_layout =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(bottom_layout[0]);
    testing_block(frame, "", false, base_layout[0]);
    testing_block(
        frame,
        match app.mode {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Visual => "Visual",
        },
        false,
        base_layout[0],
    );
    for (idx, i) in [Tab::News, Tab::Papers, Tab::Custom].iter().enumerate() {
        testing_block(
            frame,
            match i {
                Tab::News => "News",
                Tab::Papers => "Papers",
                Tab::Custom => "Custom",
            },
            if &app.tab == i { true } else { false },
            tab_layout[idx].centered_vertically(Constraint::Length(3)),
        );
    }
    frame.render_widget(search_area, sidebar_layout[0]);
    match app.tab {
        Tab::News => {
            let request_fetch_content = render_news(
                frame,
                &mut app.news_app,
                bottom_layout[1],
                sidebar_layout[1],
            );
            if request_fetch_content {
                if let Some(i) = app.news_app.sidebar.state.selected {
                    app.news_app.items[app.news_app.display_items[i]].content = Some(
                        app.tokio_runtime.block_on(
                            app.news_app
                                .fetch_content(&app.news_app.items[app.news_app.display_items[i]]),
                        ),
                    );
                }
            }
        }
        Tab::Papers => {}
        Tab::Custom => {}
    }
}

fn render_news(
    frame: &mut Frame,
    news_app: &mut News,
    bottom_layout: Rect,
    sidebar_list: Rect,
) -> bool {
    let sidebar_category_list =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(sidebar_list);
    frame.render_widget(&mut news_app.sidebar, sidebar_category_list[1]);

    let bottom =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(bottom_layout);
    let bottom_top = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .flex(Flex::SpaceBetween)
        .split(bottom[0]);

    testing_block(
        frame,
        {
            match news_app.category.get_current() {
                NewsCategoryCNA::Latest => "Category: Latest",
                NewsCategoryCNA::Asia => "Category: Asia",
                NewsCategoryCNA::Business => "Category: Business",
                NewsCategoryCNA::Singapore => "Category: Singapore",
                NewsCategoryCNA::Sports => "Category: Sports",
                NewsCategoryCNA::World => "Category: World",
                NewsCategoryCNA::Today => "Category: Today",
            }
        },
        false,
        sidebar_category_list[0],
    );

    if let Some(i) = news_app.sidebar.state.selected {
        // prevents index out of bounds when len() = 0
        // len() = 0 when no articles found
        if news_app.items.len() <= i {
            return false;
        }
        match &news_app.items[news_app.display_items[i]].content {
            Some(content) => {
                testing_block(
                    frame,
                    &news_app.items[news_app.display_items[i]].pub_date,
                    false,
                    bottom_top[0],
                );
                testing_block(
                    frame,
                    &news_app.items[news_app.display_items[i]]
                        .categories
                        .join(", "),
                    false,
                    bottom_top[1],
                );

                let viewport_height = bottom[1].height;
                let inner_width = bottom[1].width.saturating_sub(1);
                let total_lines: u16 = content
                    .iter()
                    .map(|c| count_wrapped_lines(c, inner_width))
                    .sum::<u16>()
                    + (content.len().saturating_sub(1) as u16);
                let max_scroll: u16;
                match news_app.max_scroll_offsets.get(&i) {
                    Some(scroll_offset) => {
                        max_scroll = *scroll_offset;
                    }
                    None => {
                        // Max scroll: how many lines we can scroll before the last line hits bottom
                        max_scroll = total_lines.saturating_sub(viewport_height);
                        news_app.max_scroll_offsets.insert(i, max_scroll);

                        // Clamp scroll_offset in case content changed
                        if news_app.scroll_offset > max_scroll {
                            news_app.scroll_offset = max_scroll;
                        }
                    }
                }
                let joined = content.join("\n\n");
                frame.render_widget(
                    Paragraph::new(joined)
                        .wrap(Wrap { trim: true })
                        .scroll((news_app.scroll_offset, 0))
                        .block(Block::new().borders(Borders::ALL)),
                    bottom[1],
                );

                if total_lines > viewport_height {
                    let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
                        .position(news_app.scroll_offset as usize);
                    let scrollbar =
                        Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
                    frame.render_stateful_widget(scrollbar, bottom[1], &mut scrollbar_state);
                }
                return false;
            }
            None => return true,
        }
    }
    false
}
