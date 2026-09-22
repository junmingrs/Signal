use std::time::Duration;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{self, Event, KeyCode, KeyModifiers},
    },
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use ratatui_textarea::{Key, TextArea};

use crate::{
    tui::{
        app::{App, Focused, Mode, Tab},
        tabs::{
            news::{self, Article, News, NewsCategoryKind, NewsSource},
            papers::Papers,
        },
    },
    utils::{fuzzy::fuzzy_match, article::Article, papers_model::PapersModel},
};

pub enum Message {
    GetArticles(NewsSource),
    ArticlesFetched(NewsSource, Vec<Article>),
    PapersFetched(Vec<PapersModel>),
}

fn handle_message(message: Message, app: &mut App) {
    match message {
        Message::GetArticles(category_kind) => {
            let tx = app.tx.clone();
            let news_source = app.news_app.get_current_news_source();
            let news_source_variant = news_source.source_variant;
            tokio::spawn(async move {
                let news_models = News::fetch_news(news_source_variant).await;
                tx.send(Message::ArticlesFetched(news_source, news_models))
                    .await
                    .expect("could not send message NewsFetched");
            });
        }
        Message::ArticlesFetched(news_source_index, articles) => {
            // app.news_app.items = news_models;
            app.news_app.update_source_articles(articles);
            app.news_app.reset_display_items();
            app.news_app.reload_sidebar();
            app.news_app.sidebar.state.selected = Some(0);
        }
        Message::PapersFetched(papers) => {}
    }
}

// fn handle_message(message: Message, app: &mut App, db: &Db) {
//     match message {
//         // News
//         Message::NewsFetched(items) => {
//             app.news_app.reset_display_items();
//             app.news_app.reload_sidebar();
//             app.news_app.sidebar.state.selected = Some(0);
//         }
//         Message::NewsArticlesFetched(news_models) => {
//             if news_models.len() == 0 {
//                 app.news_app.fetch_news_from_rss(app.tx.clone());
//             } else {
//                 app.news_app.items = news_models;
//                 app.news_app.reset_display_items();
//                 app.news_app.reload_sidebar();
//                 app.news_app.sidebar.state.selected = Some(0);
//             }
//         }
//         Message::NewsArticlesRequired(latest) => {
//             if latest {
//                 app.news_app.fetch_latest_news_from_db(app.tx.clone(), &db);
//             } else {
//                 app.news_app.fetch_news_from_db(app.tx.clone(), &db);
//             }
//         }
//         // Papers
//         Message::PapersRequired => {
//             app.papers_app.fetch_papers_from_db(app.tx.clone(), &db);
//         }
//         Message::PapersRSSFetched(papers_models) => {
//             db.save_papers_batch(papers_models);
//         }
//     }
// }
//
fn change_source(app: &mut App, source: NewsSource) {
    app.news_app.category.update_source(source);
    app.news_app.clear_items();
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    // setup app
    let mut app = App::new();
    // NOTE:  default category to cna latest
    let tx = app.tx.clone();
    let current_category = app.news_app.category.get_current();
    tokio::spawn(async move {
        tx.send(Message::GetArticles(current_category)).await.unwrap();
    });
    // app.papers_app.fetch_papers(app.tx.clone());
    // setup search area
    let mut search_area = TextArea::default();
    loop {
        // handle async messages
        while let Ok(msg) = app.rx.try_recv() {
            handle_message(msg, &mut app);
        }
        // handle input
        if crossterm::event::poll(Duration::from_millis(500))?
            && let Event::Key(key) = event::read()?
        {
            if key.code.is_esc() {
                app.mode = Mode::Normal
            }
            match app.mode {
                Mode::Insert => {
                    // disables newline by blocking control and enter
                    if key.modifiers == KeyModifiers::CONTROL {
                        continue;
                    }
                    // prevent reloading when search area is empty
                    if key.code == KeyCode::Enter
                        || (key.code == KeyCode::Backspace && search_area.is_empty())
                    {
                        continue;
                    }
                    search_area.input(key);
                    let mut results = fuzzy_match(
                        search_area.lines().join("").to_string(),
                        app.news_app.items.clone(),
                    );
                    results.sort_by_key(|(s, _, _)| *s);
                    results.reverse();
                    app.news_app.display_items = results.iter().map(|(_, _, i)| *i).collect();
                    if search_area.is_empty() {
                        app.news_app.reset_display_items();
                    }
                    app.news_app.reload_sidebar();
                    app.news_app.sidebar.state.selected = Some(0);
                }
                Mode::Normal => {
                    // use tab to cycle categories
                    match key.code {
                        KeyCode::Tab => match app.tab {
                            Tab::News => {
                                app.news_app.update_news_category(true);
                                app.news_app.clear_items();
                                let current_category = app.news_app.category.get_current();
                                let tx = app.tx.clone();
                                tokio::spawn(async move {
                                    tx.send(Message::GetArticles(current_category)).await.unwrap();
                                });
                                app.news_app.category.set_loaded();
                            }
                            Tab::Papers => {}
                        },
                        KeyCode::BackTab => match app.tab {
                            Tab::News => {
                                app.news_app.update_news_category(false);
                                // BUG: this logic is wrong
                                if !app.news_app.category.is_loaded() {
                                    let tx = app.tx.clone();
                                    let current_category = app.news_app.category.get_current();
                                    tokio::spawn(async move {
                                        tx.send(Message::GetArticles(current_category)).await.unwrap();
                                    });
                                }
                                // TODO:
                                // if prev cat items exist, use that
                                // else fetch new ones
                            }
                            Tab::Papers => {}
                        },
                        KeyCode::Char('1') => app.tab = Tab::News,
                        KeyCode::Char('2') => app.tab = Tab::Papers,
                        KeyCode::Char('3') => {
                            let tx = app.tx.clone();
                            let current_category = app.news_app.category.get_current();
                            tokio::spawn(async move {
                                tx.send(Message::GetArticles(current_category)).await.unwrap();
                            });
                        } // reload
                        KeyCode::Char('c') => {
                            change_source(&mut app, NewsSource::CNA);
                        }
                        KeyCode::Char('s') => {
                            change_source(&mut app, NewsSource::StraitsTimes);
                        }
                        KeyCode::Char('b') => {
                            change_source(&mut app, NewsSource::BusinessTimes);
                        }
                        KeyCode::Char('p') => {
                            if let Tab::Papers = app.tab
                                && let Some(i) = app.papers_app.sidebar.state.selected
                            {
                                webbrowser::open(
                                    &app.papers_app.items[app.papers_app.display_items[i]].link,
                                )
                                .unwrap();
                            }
                        }
                        KeyCode::Char('i') => app.mode = Mode::Insert,
                        KeyCode::Char('v') => app.mode = Mode::Visual,
                        KeyCode::Char('h') => {
                            app.focused = Focused::Left;
                            match app.tab {
                                Tab::News => app.news_app.sidebar.focused = true,
                                Tab::Papers => app.papers_app.sidebar.focused = true,
                            }
                        }
                        KeyCode::Char('l') => {
                            app.focused = Focused::Right;
                            match app.tab {
                                Tab::News => app.news_app.sidebar.focused = false,
                                Tab::Papers => app.papers_app.sidebar.focused = false,
                            }
                        }
                        KeyCode::Char('j') => match app.focused {
                            Focused::Left => match app.tab {
                                Tab::News => app.news_app.next(),
                                Tab::Papers => app.papers_app.next(),
                            },
                            Focused::Right => match app.tab {
                                Tab::News => app.news_app.scroll_down(),
                                Tab::Papers => app.papers_app.scroll_down(),
                            },
                        },
                        KeyCode::Char('k') => match app.focused {
                            Focused::Left => match app.tab {
                                Tab::News => app.news_app.previous(),
                                Tab::Papers => app.papers_app.previous(),
                            },
                            Focused::Right => match app.tab {
                                Tab::News => app.news_app.scroll_up(),
                                Tab::Papers => app.papers_app.scroll_up(),
                            },
                        },
                        KeyCode::Char('q') => break Ok(()),
                        _ => {}
                    }
                }
                Mode::Visual => {}
            }
        }
        terminal.draw(|frame| {
            render(frame, &mut app, &mut search_area);
        })?;
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

fn bordered_block(frame: &mut Frame, word: &str, selected: bool, layout: Rect) {
    frame.render_widget(
        Paragraph::new(word)
            .fg(if selected {
                Color::Yellow
            } else {
                Color::Reset
            })
            .block(Block::new().borders(Borders::ALL)),
        layout,
    );
}

fn render(frame: &mut Frame, app: &mut App, search_area: &mut TextArea) {
    let base_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(frame.area());
    let body_layout = Layout::default()
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
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(body_layout[0]);
    bordered_block(frame, "", false, base_layout[0]);
    bordered_block(
        frame,
        match app.mode {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Visual => "Visual",
        },
        false,
        base_layout[0],
    );
    for (idx, i) in [Tab::News, Tab::Papers].iter().enumerate() {
        bordered_block(
            frame,
            match i {
                Tab::News => "News",
                Tab::Papers => "Papers",
            },
            if &app.tab == i { true } else { false },
            tab_layout[idx].centered_vertically(Constraint::Length(3)),
        );
    }
    search_area.set_block(Block::default().borders(Borders::ALL).border_style(
        Style::default().fg(if let Mode::Insert = app.mode {
            Color::Yellow
        } else {
            Color::Reset
        }),
    ));
    frame.render_widget(&*search_area, sidebar_layout[0]);
    let info_layout =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(body_layout[1]);
    let pub_date_and_misc_layout =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .flex(Flex::SpaceBetween)
            .split(info_layout[0]);
    let pub_date_rect = pub_date_and_misc_layout[0];
    let misc_rect = pub_date_and_misc_layout[1];
    let content_rect = info_layout[1];
    match app.tab {
        Tab::News => {
            render_news(
                frame,
                &mut app.news_app,
                pub_date_rect,
                misc_rect,
                content_rect,
                sidebar_layout[1],
            );
        }
        Tab::Papers => {
            // render_papers(
            //     frame,
            //     &mut app.papers_app,
            //     pub_date_rect,
            //     misc_rect,
            //     content_rect,
            //     sidebar_layout[1],
            //     app.tx.clone(),
            // );
        }
    }
}

// fn render_papers(
//     frame: &mut Frame,
//     papers_app: &mut Papers,
//     pub_date_rect: Rect,
//     link_rect: Rect,
//     content_rect: Rect,
//     sidebar_list: Rect,
//     tx: Sender<Message>,
// ) {
//     frame.render_widget(&mut papers_app.sidebar, sidebar_list);
//     match papers_app.sidebar.state.selected {
//         Some(i) => {
//             let item = &papers_app.items[papers_app.display_items[i]];
//             bordered_block(frame, &item.pub_date, false, pub_date_rect);
//             bordered_block(frame, &item.link, false, link_rect);
//             let viewport_height = content_rect.height;
//             let inner_width = content_rect.width.saturating_sub(1);
//             let total_lines: u16 = count_wrapped_lines(&item.summary, inner_width)
//                 + (item.summary.chars().filter(|c| *c == '.').count() / 3) as u16;
//             let max_scroll: u16;
//             match papers_app.max_scroll_offsets.get(&i) {
//                 Some(scroll_offset) => {
//                     max_scroll = *scroll_offset;
//                 }
//                 None => {
//                     // Max scroll: how many lines we can scroll before the last line hits bottom
//                     max_scroll = total_lines.saturating_sub(viewport_height);
//                     papers_app.max_scroll_offsets.insert(i, max_scroll);
//
//                     // Clamp scroll_offset in case content changed
//                     if papers_app.scroll_offset > max_scroll {
//                         papers_app.scroll_offset = max_scroll;
//                     }
//                 }
//             }
//             frame.render_widget(
//                 Paragraph::new(item.summary.clone())
//                     .wrap(Wrap { trim: true })
//                     .scroll((papers_app.scroll_offset, 0))
//                     .block(
//                         Block::new()
//                             .borders(Borders::ALL)
//                             .border_style(Style::default().fg(if !papers_app.sidebar.focused {
//                                 Color::Yellow
//                             } else {
//                                 Color::Reset
//                             })),
//                     ),
//                 content_rect,
//             );
//
//             if total_lines > viewport_height {
//                 let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
//                     .position(papers_app.scroll_offset as usize);
//                 let scrollbar =
//                     Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
//                 frame.render_stateful_widget(scrollbar, content_rect, &mut scrollbar_state);
//             }
//         }
//         None => {
//             tokio::spawn(async move { tx.send(Message::PapersRequired).await.unwrap() });
//         }
//     }
// }

fn render_news(
    frame: &mut Frame,
    news_app: &mut News,
    pub_date_rect: Rect,
    category_rect: Rect,
    content_rect: Rect,
    sidebar_list: Rect,
) {
    let sidebar_category_list =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(sidebar_list);
    frame.render_widget(&mut news_app.sidebar, sidebar_category_list[1]);

    let category_and_source =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sidebar_category_list[0]);

    frame.render_widget(
        Paragraph::new(match news_app.category.get_current() {
            NewsCategoryKind::CNA(cna) => cna.to_string(),
            NewsCategoryKind::ST(st) => st.to_string(),
            NewsCategoryKind::BT(bt) => bt.to_string(),
        })
        .block(Block::new().borders(Borders::ALL).title("Category")),
        category_and_source[0],
    );

    frame.render_widget(
        Paragraph::new(match news_app.category.source {
            NewsSource::CNA => "CNA",
            NewsSource::StraitsTimes => "StraitsTimes",
            NewsSource::BusinessTimes => "BusinessTimes",
        })
        .block(Block::new().borders(Borders::ALL).title("Source")),
        category_and_source[1],
    );

    let (idx, total_lines, mut max_scroll) = {
        let item = news_app.get_current_news();
        bordered_block(frame, &item.pub_date, false, pub_date_rect);
        bordered_block(frame, &item.categories.join(", "), false, category_rect);
        let joined = item.content.join("\n\n");
        frame.render_widget(
            Paragraph::new(joined)
                .wrap(Wrap { trim: true })
                .scroll((news_app.scroll_offset, 0))
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(if !news_app.sidebar.focused {
                            Color::Yellow
                        } else {
                            Color::Reset
                        })),
                ),
            content_rect,
        );
        let inner_width = content_rect.width.saturating_sub(1);
        let total_lines: u16 = item
            .content
            .iter()
            .map(|c| count_wrapped_lines(c, inner_width))
            .sum::<u16>()
            + (item.content.len().saturating_sub(1) as u16);
        let idx = match news_app.sidebar.state.selected {
            Some(i) => i,
            None => 0, // BUG: default to 0 isnt a great idea
        };
        if let Some(scroll_offset) = news_app.max_scroll_offsets.get(&idx) {
            (idx, total_lines, *scroll_offset)
        } else {
            (idx, total_lines, 0_u16)
        }
    };
    let viewport_height = content_rect.height;
    if max_scroll == 0 {
        max_scroll = total_lines.saturating_sub(viewport_height);
        news_app.max_scroll_offsets.insert(idx, max_scroll);

        // Clamp scroll_offset in case content changed
        if news_app.scroll_offset > max_scroll {
            news_app.scroll_offset = max_scroll;
        }
    }
    if total_lines > viewport_height {
        let mut scrollbar_state =
            ScrollbarState::new(max_scroll as usize).position(news_app.scroll_offset as usize);
        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(scrollbar, content_rect, &mut scrollbar_state);
    }
}
