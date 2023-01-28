use crate::Result;
use reqwest::blocking::{Client, Response};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ScrapeClient {
    inner: Client,
    wait_time: Duration,
    last_waited: Instant,
}

impl ScrapeClient {
    pub fn new(wait_time: Duration) -> Result<Self> {
        Ok(Self {
            inner: Client::builder().build()?,
            wait_time,
            last_waited: Instant::now() - wait_time,
        })
    }
    pub fn get(&mut self, url: &str) -> std::result::Result<(String, String), reqwest::Error> {
        assert!(!url.contains(' '));
        let time_passed = self.last_waited - Instant::now();
        if self.wait_time > time_passed {
            thread::sleep(self.wait_time - time_passed);
        }

        let response = self.inner.get(url).send()?;
        let response = response.error_for_status()?;

        let _directed_url = response.url().as_str(); // TODO
        let file_extention = Self::get_file_extention(&response)
            .unwrap_or_else(|| panic!("Failed to get fail extention for {url}"))
            .to_owned();
        Ok((response.text()?, file_extention))
    }
    fn get_file_extention(response: &Response) -> Option<&str> {
        let content_header = response.headers().get("content-type")?;
        let content_str = content_header.to_str().ok()?;
        let content_type = content_str.split(';').next()?;
        let extention = content_type.split_once('/')?.1;
        Some(extention)
    }
}
