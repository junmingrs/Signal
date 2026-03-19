use ::sqlite::Connection;
use jiff::fmt::rfc2822::{self};

use crate::utils::cna_model::NewsModel;

pub struct db {
    connection: Connection,
}
// TODO: finish sqlite integration:
// left with loading data
impl db {
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
        let news_query =
            "INSERT INTO news (title, description, content, link, pub_date) VALUES (?, ?, ?, ?, ?)";
        let get_last_insert_id_query = "SELECT id FROM news WHERE link = ?";
        let news_category_query =
            "INSERT INTO news_category (news_id, category_name) VALUES (?, ?)";
        for news_model in news.iter() {
            let zoned =
                rfc2822::parse(&news_model.pub_date).expect("could not convert rfc2822 to zoned");
            let timestamp = zoned.timestamp().as_second();
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
    pub fn load_news(&self) {
        // TODO: right here
        // cached news should be loaded into tui first
        // prefetch next article in background
        // add bookmark feature to not auto delete after 30 days
    }
}
