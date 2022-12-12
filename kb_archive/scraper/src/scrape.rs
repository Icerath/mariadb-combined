use chrono::NaiveDate;
use log::{error, info, warn};
use reqwest::blocking::Client;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::{fs, thread};

use crate::Result;
use kb_utils::{filter_urls, url_to_path, BASE_PATH};

#[derive(Debug, Clone)]
pub struct Scraper {
    base_url: String,
    base_path: PathBuf,
    queue: VecDeque<String>,
    client: ScrapeClient,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrapeMethod {
    Standard,
    Resume,
    RecentChanges,
    Pdf,
    PdfLangs,
}

#[derive(Debug, Clone)]
struct ScrapeClient {
    inner: Client,
    wait_time: Duration,
    last_waited: Instant,
}

impl Scraper {
    pub fn new<S, P>(base_url: S, base_path: P, minimum_wait_time: Duration) -> Result<Self>
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        Ok(Self {
            base_url: base_url.into(),
            base_path: base_path.into(),
            queue: VecDeque::new(),
            client: ScrapeClient::new(minimum_wait_time)?,
        })
    }
    pub fn scrape(self, scrape_method: ScrapeMethod) -> Result<()> {
        match scrape_method {
            ScrapeMethod::Standard => self.complete_scrape(),
            ScrapeMethod::Resume => self.resume_scrape(),
            ScrapeMethod::RecentChanges => self.recent_scrape(),
            ScrapeMethod::Pdf => self.pdf_scrape(),
            ScrapeMethod::PdfLangs => todo!(),
        }
    }
}

impl Scraper {
    pub fn complete_scrape(mut self) -> Result<()> {
        self.queue.push_back("https://mariadb.com/kb/en/".into());
        let mut completed_urls = HashSet::new();
        while let Some(url) = self.queue.pop_front() {
            if completed_urls.contains(&url) {
                continue;
            }
            let Ok(html) = self.client.get(&url) else {
                warn!("Failed to request: {url}");
                completed_urls.insert(url);
                continue;
            };
            let path = url_to_path(&self.base_path, &self.base_url, &url);
            completed_urls.insert(url);
            // html
            let scraped_urls = Self::scrape_urls(&html).filter(|url| !completed_urls.contains(url));
            self.queue.extend(scraped_urls);
            let Some(_parent) = path.parent() else {
                eprintln!("Failed to Get {path:?}'s Parent");
                continue;
            };
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, html)?;
        }
        Ok(())
    }
    pub fn resume_scrape(mut self) -> Result<()> {
        self.queue.push_back("https://mariadb.com/kb/en/".into());

        let mut completed_urls = HashSet::new();
        while let Some(url) = self.queue.pop_front() {
            if completed_urls.contains(&url) {
                continue;
            }
            let path = url_to_path(&self.base_path, &self.base_url, &url);
            let html = if path.exists() {
                fs::read_to_string(&path)?
            } else {
                let Ok(html) = self.client.get(&url) else {
                    eprintln!("Failed to request {url}");
                    completed_urls.insert(url);
                    continue
                };
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, &html)?;
                html
            };
            completed_urls.insert(url);

            let scraped_urls = Self::scrape_urls(&html).filter(|url| !completed_urls.contains(url));
            self.queue.extend(scraped_urls);
        }
        Ok(())
    }
    pub fn recent_scrape(mut self) -> Result<()> {
        let updated_urls = self.get_updated_urls();
        let mut completed_urls = HashSet::new();
        for (url, _date) in updated_urls {
            let path = url_to_path(&self.base_path, &self.base_url, &url);
            let dir_path = &path.parent().unwrap().join(path.file_stem().unwrap());
            if !completed_urls.contains(&url) {
                let html = self.client.get(&url)?;
                completed_urls.insert(url);
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, &html)?;
                log::info!("Wrote to {path:?}");
            }
            if dir_path.exists() {
                self.recursive_update(dir_path, &mut completed_urls)?;
            }
        }
        Ok(())
    }
    fn pdf_scrape(mut self) -> Result<()> {
        let csv = kb_utils::kb_urls::read_kb_urls();
        for row in csv.into_iter().filter(|row| row.pdf_include == 1) {
            let url = row.url;
            let Ok(html) = self.client.get(&url) else {
                error!("Failed to request: {url}");
                continue;
            };
            let path = url_to_path(&self.base_path, &self.base_url, &url);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &html)?;
        }
        Ok(())
    }
    ///
    fn recursive_update(&mut self, dir: &Path, completed_urls: &mut HashSet<String>) -> Result<()> {
        for path in dir.read_dir()? {
            let path = path.unwrap().path();
            if path.is_dir() {
                self.recursive_update(&path, completed_urls)?;
            } else {
                let url = Self::path_to_url(&path);
                if !completed_urls.contains(&url) {
                    let html = self.client.get(&url)?;
                    completed_urls.insert(url);
                    fs::create_dir_all(path.parent().unwrap())?;
                    fs::write(&path, &html)?;
                    log::info!("Wrote to {path:?}");
                }
            }
        }
        Ok(())
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
            .map(|(url, date)| (kb_utils::format_url(url), date))
            .filter(|(url, _date)| kb_utils::valid_url(url))
    }
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
    fn get_updated_urls(&mut self) -> Vec<(String, NaiveDate)> {
        let last_updated: NaiveDate = fs::read_to_string("last_updated.txt")
            .expect("Failed to read 'last_updated.txt'")
            .parse()
            .expect("Invalid content in 'last_updated.txt'");

        let mut page_num = 1;
        let mut recent_articles = vec![];
        loop {
            let url = format!("https://mariadb.com/kb/+changes/?p={page_num}");
            let html = self
                .client
                .get(&url)
                .unwrap_or_else(|_| panic!("Failed to request: '{url}'"));
            let all_articles: Vec<_> = Self::scrape_recent_article_urls(&html).collect();
            let num_all_articles = all_articles.len();
            let new_articles: Vec<_> = all_articles
                .into_iter()
                .filter(|(_url, date)| *date >= last_updated)
                .collect();
            if num_all_articles > new_articles.len() {
                break;
            }
            recent_articles.extend(new_articles);
            page_num += 1;
        }

        recent_articles
    }
}

impl ScrapeClient {
    pub fn new(wait_time: Duration) -> Result<Self> {
        Ok(Self {
            inner: Client::builder().build()?,
            wait_time,
            last_waited: Instant::now() - wait_time,
        })
    }
    pub fn get(&mut self, url: &str) -> Result<String> {
        let time_passed = self.last_waited - Instant::now();
        if self.wait_time > time_passed {
            thread::sleep(self.wait_time - time_passed);
        }

        let response = match self.inner.get(url).send() {
            Ok(response) => response,
            Err(err) => match () {
                _ if err.is_connect() => panic!("Connection Error"),
                _ if err.is_request() => panic!("Request Error"),
                _ => return Err(Box::new(err)),
            },
        };
        let response = response.error_for_status()?;

        let _directed_url = response.url().as_str(); // TODO
        info!(
            "Requested: {}",
            url.trim_start_matches("https://mariadb.com/kb/")
        );
        Ok(response.text()?)
    }
}
