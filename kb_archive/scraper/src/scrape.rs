use chrono::NaiveDate;
use lazy_regex::regex;
use log::{error, info, warn};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};

use crate::format_url::filter_urls;
use crate::scrape_client::ScrapeClient;
use crate::{Result, BASE_PATH, CONNECTION_WAIT_TIME, URL_LOCATIONS_PATH, URL_LOCATIONS_SEP};

#[derive(Debug, Clone)]
pub(crate) struct Scraper {
    queue: VecDeque<String>,
    client: ScrapeClient,
    url_locations: HashMap<String, PathBuf>,
    ignore_existing: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrapeMethod {
    Standard,
    RecentChanges,
    Pdf,
    PdfLangs,
}

impl Scraper {
    pub fn new(minimum_wait_time: Duration, ignore_existing: bool) -> Result<Self> {
        Ok(Self {
            queue: VecDeque::new(),
            client: ScrapeClient::new(minimum_wait_time)?,
            url_locations: read_url_locations(),
            ignore_existing,
        })
    }
    pub fn scrape(mut self, scrape_method: ScrapeMethod) {
        match scrape_method {
            ScrapeMethod::Standard => self.standard_scrape(),
            ScrapeMethod::RecentChanges => self.recursive_recent_scrape(),
            ScrapeMethod::Pdf => self.pdf_scrape(),
            ScrapeMethod::PdfLangs => self.pdf_langs_scrape(),
        };
        // Pdf and Pdflangs don't completely update the archive.
        if !matches!(scrape_method, ScrapeMethod::Pdf | ScrapeMethod::PdfLangs) {
            self.write_url_locations()
                .expect("Failed to write url locations");
        }
    }
}

impl Scraper {
    pub fn write_url_locations(&mut self) -> Result<()> {
        let contents = self
            .url_locations
            .iter()
            .map(|(k, v)| format!("{k} {}", format!("{v:?}").trim_matches('"')))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(URL_LOCATIONS_PATH, contents)?;
        Ok(())
    }
    pub fn get_and_write(&mut self, url: &str) -> Result<(String, PathBuf)> {
        info!("Requesting: {url}");
        let mut count = 0;
        let (html, file_extention) = loop {
            count += 1;
            match self.client.get(url) {
                Ok(content) => break content,
                Err(err) => {
                    if err.is_connect() || err.is_request() {
                        thread::sleep(CONNECTION_WAIT_TIME);
                        warn!("Connection Failed: '{url}' - {count}");
                    } else {
                        return Err(Box::new(err));
                    }
                }
            }
        };
        let path = url_to_path(url, &file_extention);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, &html)?;
        self.url_locations
            .insert(url.trim_end_matches('/').to_owned(), path.clone());
        self.write_url_locations()?;
        Ok((html, path))
    }
    pub fn standard_scrape(&mut self) {
        self.queue.push_back("https://mariadb.com/kb/en/".into());
        let mut completed_urls = HashSet::new();
        while let Some(url) = self.queue.pop_front() {
            if completed_urls.contains(&url) {
                continue;
            }
            let Ok((html, _path)) = self.get_and_write(&url) else {
                warn!("Failed to request: {url}");
                completed_urls.insert(url);
                continue;
            };
            completed_urls.insert(url);
            // html
            let scraped_urls = Self::scrape_urls(&html).filter(|url| !completed_urls.contains(url));
            self.queue.extend(scraped_urls);
        }
    }
    pub fn recursive_recent_scrape(&mut self) {
        let updated_urls = self.get_updated_urls();
        let mut completed_urls = HashSet::new();
        for (url, _date) in updated_urls {
            if completed_urls.contains(&url) {
                continue;
            }
            let (_html, path) = self
                .get_and_write(&url)
                .unwrap_or_else(|_| panic!("todo!: {url}"));
            completed_urls.insert(url);
            for subpath in Self::get_file_recursive_subpaths(&path) {
                let url = Self::path_to_url(&subpath);
                if completed_urls.contains(&url) {
                    continue;
                }
                let (_html, _path) = self
                    .get_and_write(&url)
                    .unwrap_or_else(|_| panic!("todo!: {url}"));
                completed_urls.insert(url);
            }
        }
    }
    fn pdf_scrape(&mut self) {
        let csv = crate::kb_urls::read_kb_urls();
        for row in csv {
            if row.pdf_include != 1
                || (self.ignore_existing
                    && self
                        .url_locations
                        .contains_key(row.url.trim_end_matches('/')))
            {
                continue;
            }
            let Ok((_html, _path)) = self.get_and_write(&row.url) else {
                error!("Failed to request: {}", &row.url);
                continue;
            };
        }
    }
    fn pdf_langs_scrape(&mut self) {
        let csv = crate::kb_urls::read_kb_urls();
        let mut urls = csv
            .into_iter()
            .filter(|row| row.pdf_include != 0)
            .map(|row| row.url)
            .collect::<Vec<_>>();
        urls.dedup();
        for url in &urls {
            let Ok((html, _path)) = self.get_and_write(url) else {
                error!("Failed to request: {}", url);
                continue;
            };
            for url in Self::get_lang_urls(&html) {
                let Ok((_html, _path))  = self.get_and_write(&url) else {
                    error!("Failed to request: {}", &url);
                    continue;
                };
            }
        }
    }
    fn get_lang_urls(html: &str) -> Vec<String> {
        let find_body = regex!(
            r#"<h\d> *Localized Versions *</h\d> *</div> *<div> *<ul>( *<li><a href="[^"]+">[^<]*</a> *\[[\w\d-]+\] *</li>)+"#
        );
        let find_hrefs = regex!(r#"href="([^"]+)""#);

        let html = html.replace('\n', "");
        let Some(body) = find_body.find(&html) else { return vec![] };
        let hrefs = find_hrefs.captures_iter(body.as_str());
        filter_urls(hrefs.map(|href| href[1].to_owned())).collect()
    }
    fn path_to_url(path: &Path) -> String {
        let path = path.with_extension("");
        let str = path
            .to_str()
            .unwrap()
            .trim_start_matches(BASE_PATH)
            .replace('\\', "/");
        String::from("https://mariadb.com/kb/") + &str
    }
}

impl Scraper {
    fn scrape_urls(html: &str) -> impl Iterator<Item = String> + '_ {
        let scraped_urls = Self::scrape_raw_urls(html);
        filter_urls(scraped_urls)
    }

    fn scrape_raw_urls(html: &str) -> impl Iterator<Item = String> + '_ {
        let re = lazy_regex::regex!(r#"="(?:(?:https://)?mariadb.com)?/?kb/([^"]*)""#);
        re.captures_iter(html).map(|cap| cap[1].to_owned())
    }
    fn scrape_recent_article_urls(html: &str) -> impl Iterator<Item = (String, NaiveDate)> + '_ {
        let scraped_urls = Self::scrape_recent_article_urls_raw(html);
        scraped_urls
            .map(|(url, date)| (crate::format_url::format_url(url), date))
            .filter(|(url, _date)| crate::format_url::valid_url(url))
    }
    // Reads the urls and their dates for each article on a recent changes page
    /// Currently Bugged: Will fix soon.
    fn scrape_recent_article_urls_raw(
        html: &str,
    ) -> impl Iterator<Item = (String, NaiveDate)> + '_ {
        let re_urls = lazy_regex::regex!(
            r#"<li class="history_item" value="\d+">Article <a href="(?:(?:https://)?mariadb.com)?/?kb/([^"]*)">"#
        );
        let re_dates = lazy_regex::regex!(r#"<span class="datetime" title="([^"]+)">"#);
        let urls = re_urls.captures_iter(html).map(|cap| cap[1].to_owned());
        let dates = re_dates
            .captures_iter(html)
            .map(|cap| cap[1].to_owned())
            .map(|s| {
                let (first, _second) = s.split_once(' ').unwrap();
                first.parse::<NaiveDate>().unwrap()
            });
        urls.zip(dates)
    }
    /// Finds all the urls that have been updated since the last time the scraper was updated.
    fn get_updated_urls(&mut self) -> Vec<(String, NaiveDate)> {
        let last_updated: NaiveDate = fs::read_to_string("last_updated.txt")
            .unwrap_or_else(|_| NaiveDate::MIN.to_string())
            .parse()
            .unwrap_or(NaiveDate::MIN);
        dbg!(last_updated);
        let mut recent_articles = vec![];
        for page_num in 1.. {
            let url = format!("https://mariadb.com/kb/+changes/?p={page_num}");
            let (html, _path) = self.get_and_write(&url).expect("Failed to Request");
            let all_articles: Vec<_> = Self::scrape_recent_article_urls(&html).collect();
            let num_all_articles = all_articles.len();
            let new_articles: Vec<_> = all_articles
                .into_iter()
                .filter(|(_url, date)| *date >= last_updated)
                .collect();
            let new_articles_len = new_articles.len();
            recent_articles.extend(new_articles);
            if num_all_articles > new_articles_len {
                break;
            }
        }
        recent_articles
    }
    fn get_file_recursive_subpaths(input: &Path) -> Vec<PathBuf> {
        let mut output = vec![];
        let input = input.with_extension("");
        if !input.exists() {
            return output;
        }
        for path in input.read_dir().unwrap() {
            let path = path.unwrap().path();
            if path.is_dir() {
                Self::get_file_recursive_subpaths(&path);
            } else {
                output.push(path);
            }
        }
        output
    }
}

fn read_url_locations() -> HashMap<String, PathBuf> {
    fs::read_to_string(URL_LOCATIONS_PATH)
        .unwrap_or_else(|_| String::new())
        .lines()
        .map(|line| {
            line.split_once(URL_LOCATIONS_SEP)
                .expect("Invalid url locations")
        })
        .map(|(a, b)| (a.to_owned(), PathBuf::from(b)))
        .collect()
}

fn url_to_path(url: &str, file_extention: &str) -> PathBuf {
    let base_path = PathBuf::from(BASE_PATH);
    let url_suffix = get_url_suffix(url);
    base_path.join(url_suffix).with_extension(file_extention)
}

fn get_url_suffix(mut url: &str) -> &str {
    if let Some(idx) = url.find('?') {
        url = &url[..idx];
    }
    url.trim_start_matches("https://")
        .trim_start_matches("mariadb.com/")
        .trim_start_matches("kb/")
}
