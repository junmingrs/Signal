use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};

pub fn get_default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/rss+xml, application/xml;q=0.9, */*;q=0.8"),
    );
    headers
}
