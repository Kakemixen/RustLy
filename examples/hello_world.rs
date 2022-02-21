use rustly;
use rustly::log::*;
use std::thread;
use std::time::Duration;

fn main()
{
	log_init();
	let window = rustly::window::create_window();
	let event_handler = rustly::window::get_empty_event_loop();
	window.run(event_handler);
}
