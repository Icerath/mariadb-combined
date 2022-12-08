mod logger;
mod scrape;

use chrono::Utc;
use clap::{arg, Command};
use kb_utils::{BASE_PATH, BASE_URL};
use log::info;
pub use scrape::ScrapeMethod;
use std::{fs, time::Duration};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_WAIT_TIME: Duration = Duration::from_millis(500);

fn main() {
    logger::init();
    let args = parse_args();
    if args.clear {
        clear_html();
    }
    let scraper = scrape::Scraper::new(BASE_URL, BASE_PATH, DEFAULT_WAIT_TIME).unwrap();
    scraper.scrape(args.scrape_method).unwrap();
    set_last_updated();
}

#[derive(Debug)]
struct AppArgs {
    scrape_method: ScrapeMethod,
    clear: bool,
}

fn parse_args() -> AppArgs {
    let matches = Command::new("KbScraper")
        .arg(arg!([scrape_method] "[standard|resume|recent|pdf|pdf_langs]"))
        .arg(arg!(-c --clear "Clears out the HTML Directory"))
        .get_matches();

    let scrape_method_string = matches
        .get_one::<String>("scrape_method")
        .map_or_else(|| "standard", String::as_str);

    let scrape_method = match scrape_method_string.to_lowercase().as_str() {
        "" | "standard" => ScrapeMethod::Standard,
        "resume" => ScrapeMethod::Resume,
        "recent" => ScrapeMethod::RecentChanges,
        "pdf" => ScrapeMethod::Pdf,
        "pdf_langs" => ScrapeMethod::PdfLangs,
        other => panic!("Invalid Scrape Method: '{other}'"),
    };

    let clear = matches.get_one::<bool>("clear").copied().unwrap_or(false);

    AppArgs {
        scrape_method,
        clear,
    }
}

fn clear_html() {
    info!("Clearing {BASE_PATH}");
    fs::remove_dir_all(BASE_PATH).unwrap();
}

fn set_last_updated() {
    let text = Utc::now().date_naive().to_string();
    fs::write("last_updated.txt", text).unwrap();
}
