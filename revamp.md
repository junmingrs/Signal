app.news


// NOTE:
// use app.news.sources.(get category).
News { // DONE
    sources: Vec<NewsSource>
    source_index: usize,
    sidebar: Sidebar

    // NOTE: deal with scroll offsets later
    selected: Selected (the struct given by ListState i forgot)
}

Sidebar { // DONE
    display_titles: Vec<String>,
    state: ListState,
    focused: bool,
}

enum NewsSourceEnum { 
    CNA(NewsCategoryCNA),
    ST(NewsCategoryST),
    BT(NewsCategoryBT),
}

enum NewsCategoryCNA { // DONE
    Latest(Vec<Article>),
    Asia(Vec<Article>),
    Business(Vec<Article>),
    Singapore(Vec<Article>),
    Sports(Vec<Article>),
    World(Vec<Article>),
    Today(Vec<Article>),
}

NewsSource { // DONE
    source_name: NewsSourceEnum
    category_index: usize
}

Article { // DONE
    title: String,
    content: Vec<String>,
    max_scroll: u32?
}

