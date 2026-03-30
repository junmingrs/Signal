use crate::{services::cna::NewsCategoryCNA, tui::tabs::news::NewsCategoryKind};

pub fn is_latest(news_category_kind: NewsCategoryKind) -> bool {
    matches!(
        news_category_kind,
        NewsCategoryKind::CNA(NewsCategoryCNA::Latest)
    )
}
