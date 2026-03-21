use ::sqlite::Connection;

use crate::utils::{
    news_model::NewsModel,
    time_formatter::{self, custom_time_to_unix},
};

pub struct Db {
    connection: Connection,
}

// TODO: bugs to fix:
// move on to async fetching and saving
impl Db {
    pub fn new() -> Self {
        let connection = sqlite::open("db.sqlite").expect("unable to create db");
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS news (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    description TEXT NOT NULL, 
                    content TEXT,
                    link TEXT UNIQUE NOT NULL,
                    pub_date TEXT NOT NULL
                );",
            )
            .expect("create news table failed");
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS news_category (
                    news_id INTEGER,
                    category_name TEXT,
                    CONSTRAINT NEWS_CATEGORY_PK PRIMARY KEY (news_id, category_name)
                )",
            )
            .expect("create news category table failed");
        Self { connection }
    }
    pub fn save_news(&self, news: Vec<NewsModel>) {
        let news_query = "INSERT OR IGNORE INTO news (title, description, content, link, pub_date) VALUES (?, ?, ?, ?, ?)";
        let get_last_insert_id_query = "SELECT id FROM news WHERE link = ?";
        let news_category_query =
            "INSERT OR IGNORE INTO news_category (news_id, category_name) VALUES (?, ?)";
        for news_model in news.iter() {
            let timestamp = custom_time_to_unix(&news_model.pub_date);
            if let None = news_model.content {
                return;
            }
            let content = news_model
                .content
                .as_ref()
                .map(|c| c.join("\n"))
                .unwrap_or_default();
            let mut news_statement = self.connection.prepare(news_query).unwrap();
            news_statement.bind((1, news_model.title.as_str())).unwrap();
            news_statement
                .bind((2, news_model.description.as_str()))
                .unwrap();
            news_statement.bind((3, content.as_str())).unwrap();
            news_statement.bind((4, news_model.link.as_str())).unwrap();
            news_statement.bind((5, timestamp)).unwrap();
            news_statement.next().unwrap();
            let mut last_insert_id_statement =
                self.connection.prepare(get_last_insert_id_query).unwrap();
            last_insert_id_statement
                .bind((1, news_model.link.as_str()))
                .unwrap();
            last_insert_id_statement.next().unwrap();
            let id = last_insert_id_statement.read::<i64, _>("id").unwrap();
            for category in news_model.categories.iter() {
                let mut news_category_statement =
                    self.connection.prepare(news_category_query).unwrap();
                news_category_statement.bind((1, id)).unwrap();
                news_category_statement
                    .bind((2, category.as_str()))
                    .unwrap();
                news_category_statement.next().unwrap();
            }
        }
    }
    pub fn fetch_news(&self, limit: i64) -> Vec<NewsModel> {
        // TODO: right here
        // cached news should be loaded into tui first
        // prefetch next article in background
        // add bookmark feature to not auto delete after 30 days
        let mut news: Vec<NewsModel> = Vec::new();
        let fetch_news_query = "SELECT * FROM news LIMIT ?";
        let fetch_categories_query = "SELECT * FROM news_category WHERE news_id = ?";
        for news_row in self
            .connection
            .prepare(fetch_news_query)
            .unwrap()
            .into_iter()
            .bind((1, limit))
            .unwrap()
            .map(|row| row.unwrap())
        {
            let id = news_row.read::<i64, _>("id");
            let title = news_row.read::<&str, _>("title").to_string();
            let description = news_row.read::<&str, _>("description").to_string();
            let fetched_content = news_row.read::<&str, _>("content").to_string();
            let content = Some(
                fetched_content
                    .split("\n")
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
            );
            let link = news_row.read::<&str, _>("link").to_string();
            let pub_date = news_row.read::<i64, _>("pub_date");
            let mut categories: Vec<String> = Vec::new();
            for category_row in self
                .connection
                .prepare(fetch_categories_query)
                .unwrap()
                .into_iter()
                .bind((1, id))
                .unwrap()
                .map(|r| r.unwrap())
            {
                categories.push(category_row.read::<&str, _>("category_name").to_string());
            }
            let formatted_pub_date = time_formatter::unix_to_custom_time(pub_date);
            news.push(NewsModel {
                title,
                description,
                content,
                link,
                pub_date: formatted_pub_date,
                categories,
            });
        }
        news
    }
    // pub fn fetch_news_without_content(&self) -> Vec<NewsModel> {
    //     let mut news_without_content: Vec<NewsModel> = Vec::new();
    //     let content_exist_query = "SELECT * FROM news WHERE content IS NULL";
    //     let fetch_categories_query = "SELECT * FROM news_category WHERE news_id = ?";
    //     for row in self
    //         .connection
    //         .prepare(content_exist_query)
    //         .unwrap()
    //         .into_iter()
    //         .map(|row| row.unwrap())
    //     {
    //         let id = row.read::<i64, _>("id");
    //         let title = row.read::<&str, _>("title").to_string();
    //         let description = row.read::<&str, _>("description").to_string();
    //         let link = row.read::<&str, _>("link").to_string();
    //         let pub_date = row.read::<i64, _>("pub_date");
    //         let formatted_date = unix_to_custom_time(pub_date);
    //         let mut categories: Vec<String> = Vec::new();
    //         for category_row in self
    //             .connection
    //             .prepare(fetch_categories_query)
    //             .unwrap()
    //             .into_iter()
    //             .bind((1, id))
    //             .unwrap()
    //             .map(|r| r.unwrap())
    //         {
    //             categories.push(category_row.read::<&str, _>("category_name").to_string());
    //         }
    //         news_without_content.push(NewsModel {
    //             title,
    //             description,
    //             content: None,
    //             link,
    //             pub_date: formatted_date,
    //             categories,
    //         });
    //     }
    //     news_without_content
    // }
    // pub fn bulk_update_news_content(&self, news_with_content: Vec<NewsModel>) {
    //     let news_without_content = self.fetch_news_without_content();
    // }
    pub fn check_news_exist(&self, news: &NewsModel) -> bool {
        let check_query = "SELECT 1 FROM news WHERE title = ?";
        let mut statement = self.connection.prepare(check_query).unwrap();
        statement.bind((1, news.title.as_str())).unwrap();
        match statement.next().unwrap() {
            sqlite::State::Row => true,
            sqlite::State::Done => false,
        }
    }
}
