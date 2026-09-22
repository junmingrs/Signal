use crate::utils::article::Article;

pub fn fuzzy_score(query: &String, cna_model: &Article) -> i32 {
    // should be u16
    let query = query.to_lowercase();
    let text = cna_model.title.to_lowercase();
    let (mut qi, mut score, mut streak) = (0, 0, 0);
    for char in text.chars().into_iter() {
        if qi < query.len() && char == query.chars().nth(qi).unwrap() {
            // mysterious unwrap here
            qi += 1;
            streak += 1;
            score += 10 + streak;
        } else {
            streak = 0;
        }
    }
    if qi != query.len() {
        return 0;
    }
    return score;
}

pub fn fuzzy_match(query: String, choices: Vec<Article>) -> Vec<(i32, Article, usize)> {
    let mut results = Vec::<(i32, Article, usize)>::new();
    for (i, choice) in choices.iter().enumerate() {
        let s = fuzzy_score(&query, choice);
        if s > 0 {
            results.push((s, choice.clone(), i));
        }
    }
    results
}
