pub mod kb_urls;

use std::path::PathBuf;

pub const IGNORED_URL_SEGMENTS: &[&str] = &[
    "/+change_order",
    "/add/",
    "/ask/",
    "/remove/",
    "/+history/",
    "/+translate",
    "/+flag",
    "/post/",
];
pub const IGNORED_URL_SUFFIXES: &[&str] = &[
    "+flag", "/ask", "+search", "/post", "/remove", "/flag", "/add",
];

pub const BASE_URL: &str = "https://mariadb.com/kb/";
pub const BASE_PATH: &str = "../HTML/";

pub fn url_to_path<P: Into<PathBuf>>(base_path: P, url_prefix: &str, url: &str) -> PathBuf {
    let url_suffix = url.trim_start_matches(url_prefix).trim_matches('/');
    let path: PathBuf = base_path.into().join(url_suffix);
    match path.extension() {
        Some(_) => path,
        None => path.with_extension("html"),
    }
}

pub fn format_url<S: AsRef<str>>(url: S) -> String {
    let mut suffix = url.as_ref();
    for symbol in ['#', '?'] {
        if let Some(idx) = suffix.find(symbol) {
            suffix = &suffix[..idx];
        }
        debug_assert!(!suffix.contains(symbol));
    }

    let url = suffix
        .trim_start_matches('/')
        .trim_start_matches("kb/")
        .trim_start_matches("mariadb.com/kb/")
        .trim_start_matches("https://mariadb.com/kb/")
        .trim();

    String::from("https://mariadb.com/kb/") + url
}

pub fn valid_url(url: &str) -> bool {
    debug_assert_eq!(url, &format_url(url));
    if IGNORED_URL_SEGMENTS
        .iter()
        .any(|segment| url.contains(segment))
    {
        return false;
    };
    if IGNORED_URL_SUFFIXES
        .iter()
        .any(|suffix| url.ends_with(suffix))
    {
        return false;
    }
    !url.trim_start_matches("https://mariadb.com/kb/").is_empty()
}

pub fn filter_urls(urls: impl Iterator<Item = String>) -> impl Iterator<Item = String> {
    urls.map(format_url).filter(|url| valid_url(url))
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_format_url_bulk() {
        let inputs_and_outputs = include_str!("../../url_tests.txt");
        for content in inputs_and_outputs.lines() {
            let (input, output) = content.split_once(' ').unwrap();
            assert_eq!(format_url(input.trim()), output.trim());

        }
    }
}