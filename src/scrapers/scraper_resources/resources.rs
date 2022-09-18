use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;

use chrono::{Month, NaiveDate, NaiveDateTime, NaiveTime};
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::info;

use crate::error::NewsError;
use crate::error::NewsError::{BadSelector, MonthParse, SelectedNothing};
use crate::scrapers::scraper_resources::html_util::{ElemUtil, format_selector, HtmlUtil};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
/// Defines the types of pages where news come from
pub enum ScrapeType {
	Forum,
	Main,
	Changelog,
}

impl Display for ScrapeType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ScrapeType::Forum => {
				write!(f, "Forum news")
			}
			ScrapeType::Main => {
				write!(f, "News")
			}
			ScrapeType::Changelog => {
				write!(f, "Changelog")
			}
		}
	}
}

pub async fn request_html(url: &str) -> Result<Html, NewsError> {
	info!("Fetching data from {}", &url);

	let client = Client::builder()
		.timeout(Duration::from_secs(5))
		.build()?;
	let raw_html = client.get(url).send().await?;
	let text = raw_html.text().await?;
	Ok(Html::parse_document(text.as_str()))
}

pub fn get_listed_links(scrape_type: ScrapeType, html: &Html) -> Result<Vec<String>, NewsError> {
	return match scrape_type {
		ScrapeType::Changelog | ScrapeType::Main => {
			let sel_text = if scrape_type == ScrapeType::Main {
				// ---------------------------------------------------------↓ I dont make the rules ¯\_(ツ)_/¯
				"#bodyRoot > div.content > div:nth-child(2) > div:nth-child(2) > div > section > div > div.showcase__content-wrapper > div.showcase__item"
			} else {
				// ---------------------------------------------------------↓ I dont make the rules ¯\_(ツ)_/¯
				"#bodyRoot > div.content > div:nth-child(2) > div:nth-child(3) > div > section > div > div.showcase__content-wrapper > div.showcase__item"
			};
			let sel = format_selector(sel_text)?;

			let date_sel_text = "div.widget__content > ul > li".to_owned();

			let selected = html.select(&sel);
			let mut res = vec![];
			for item in selected {
				if let Ok(url) = item.select_first("a", &scrape_type.to_string())?.select_attribute("href", &scrape_type.to_string()) {
					res.push(url.to_owned());
				}
			}
			Ok(res)
		}
		ScrapeType::Forum => {
			static SEL_TEXT: &str = "body > main > div > div > div > div:nth-child(2) > div > ol > li";
			let sel = Selector::parse(SEL_TEXT).map_err(|_| BadSelector(SEL_TEXT.to_owned()))?;

			let lower_url_test = "div > h4 > div > a".to_owned();
			let lower_url = Selector::parse(&lower_url_test).map_err(|_| BadSelector(lower_url_test.clone()))?;

			let date_sel_text: String = "div > div > time".to_owned();
			let date_sel = Selector::parse(&date_sel_text).map_err(|_| NewsError::BadSelector(date_sel_text.clone()))?;

			let selected = html.select(&sel);
			let mut res = vec![];
			for item in selected {
				if let Some(url_elem) = item.select(&lower_url).next() {
					if let Some(url) = url_elem.value().attr("href") {
						if let Some(date_str) = item.select(&date_sel).next().ok_or_else(|| SelectedNothing(date_sel_text.clone(), item.inner_html()))?.value().attr("datetime") {
							res.push(url.to_owned());
						}
					}
				}
			}
			Ok(res)
		}
	};
}

pub fn format_into_final_url(top_url: &str, selection: ScrapeType) -> String {
	match selection {
		ScrapeType::Main | ScrapeType::Changelog => {
			format!("https://warthunder.com{}", top_url)
		}
		ScrapeType::Forum => {
			top_url.to_owned()
		}
	}
}