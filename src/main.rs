#![feature(if_let_guard)]
#![feature(once_cell)]
#![allow(clippy::module_name_repetitions)]

use std::{fs, io};
use std::error::Error;
use std::io::stdout;
use std::process::exit;

use lazy_static::{initialize, lazy_static};
use tracing::{error, Level};

use crate::fetch_loop::fetch_loop;
use crate::json::webhooks::CrashHook;
use crate::json::webhooks::WebhookAuth;
use crate::menu_options::{add_webhook, remove_webhook, test_hook};

use tracing_appender::rolling;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::filter::EnvFilter;

mod webhook_handler;
mod scrapers;
mod json;
mod menu_options;
mod fetch_loop;
mod embed;
mod error;
mod timeout;
mod statistics;

const RECENT_PATH: &str = "assets/sources.json";
const TOKEN_PATH: &str = "assets/discord_token.json";

pub const HANDLE_RESULT_FN: fn(Result<(), Box<dyn Error>>) = |e: Result<(), Box<dyn Error>>| {
	match e {
		Ok(_) => {}
		Err(e) => {
			error!(e);
			panic!("{}", e);
		}
	}
};

lazy_static! {
	pub static ref WEBHOOK_AUTH: WebhookAuth = {
		let raw = fs::read(TOKEN_PATH).unwrap();
		let json: WebhookAuth = serde_json::from_slice(&raw).unwrap();
		json
	};
	pub static ref PANIC_INFO: CrashHook = {
		WEBHOOK_AUTH.crash_hook[0].clone()
	};
}

#[tokio::main]
async fn main() {
	// Loads statics
	initialize(&WEBHOOK_AUTH);
	initialize(&PANIC_INFO);

	let mut line = String::new();
	let mut hooks = true;

	println!("Please select a start profile:\n\
	1. Regular initialization\n\
	2. Boot without sending hooks\n\
	3. Add new webhook-client\n\
	4. Remove a webhook\n\
	5. Without hooks or file-verification\n\
	6. Test webhook client / channel");
	io::stdin().read_line(&mut line).expect("failed to read from stdin");

	// LOGGING CONVENTION
	// Trace - Typically unused
	// Info - Used for things that happen in guaranteed intervals
	// Warn - Used for irregular occurrences such as finding news
	// Error - Any (un)recoverable error blocking part of regular execution or halting it entirely

	let debug_file = rolling::daily("./log/debug", "debug").with_max_level(Level::INFO);
	let warn_file = rolling::never("./log/warning", "warnings").with_min_level(Level::WARN);
	let all_files = debug_file.and(warn_file);

	let env_filter = EnvFilter::from_default_env()
		.add_directive(Level::INFO.into())
		.add_directive("wt_event_handler=debug".parse().unwrap());


	tracing_subscriber::fmt()
		.with_thread_names(true)
		.with_env_filter(env_filter)
		.with_file(true)
		.with_line_number(true)
		.with_writer(stdout.and(all_files))
		.with_ansi(false)
		.init();

	tracing::info!("Started program");

	match line.trim() {
		"1" => {}
		"2" => { hooks = false; }
		"3" => { HANDLE_RESULT_FN(add_webhook().await) }
		"4" => { HANDLE_RESULT_FN(remove_webhook()) }
		"5" => {
			hooks = false;
		}
		"6" => {
			hooks = false;
			HANDLE_RESULT_FN(test_hook().await);
		}
		_ => {
			tracing::error!("Bad options - aborting");
			exit(1);
		}
	}

	fetch_loop(hooks).await;
}