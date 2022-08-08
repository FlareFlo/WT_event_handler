use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;

use chrono::Local;

use crate::logging::{LogLevel, print_log};
use crate::RECENT_PATH;
use crate::scrapers::scraper_resources::resources::ScrapeType;

#[derive(Default, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct Recent {
	pub meta: Meta,
	pub sources: Vec<Channel>,
}

#[derive(Default, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct Meta {
	pub timestamp: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct Channel {
	pub name: String,
	pub domain: String,
	pub scrape_type: ScrapeType,
	pub selector: String,
	pub pin: String,
	pub old_urls: HashSet<String>,
}

impl Channel {
	pub fn is_new(&self, value: &str) -> bool {
		if self.old_urls.get(&value.to_owned()).is_some() {
			print_log("Content was recently fetched and is not new", LogLevel::Info);
			false
		} else {
			print_log("New post found, hooking now", LogLevel::Warning);
			true
		}
	}
	pub fn store_recent(&mut self, value: &str) {
		self.old_urls.insert(value.to_owned());
	}
}

impl Recent {
	pub fn save(&mut self) {
		self.update_timestamp();

		let write = serde_json::to_string_pretty(self).unwrap();
		fs::write(RECENT_PATH, write).expect("Couldn't write to recent file");
		print_log("Saved recent to file", LogLevel::Warning);
	}
	fn update_timestamp(&mut self) {
		self.meta.timestamp = u64::try_from(Local::now().timestamp()).unwrap();
	}
	pub fn read_latest() -> Self {
		let cache_raw_recent = fs::read_to_string(RECENT_PATH).expect("Cannot read file");
		let recent: Self = serde_json::from_str(&cache_raw_recent).expect("Json cannot be read");
		recent
	}
}
