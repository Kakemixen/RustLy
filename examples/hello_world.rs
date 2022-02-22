use rustly::log::*;
use rustly::{events, window};
use std::sync::Arc;
use std::thread;
//use std::time::Duration;

fn main()
{
	log_init();
	let window = window::create_window();

	let channel = Arc::new(events::SyncEventChannel::<window::LyWindowEvent>::new());

	let c = Arc::clone(&channel);
	thread::spawn(move || {
		let reader = c.get_reader();

		loop {
			let mut should_exit = false;

			// need this for some reason, or it will drop events
			// TODO why?
			// thread::sleep(Duration::from_millis(1));
			// wait_new solves it here, but not root cause
			reader.wait_new();

			info!("flushing!");
			reader.flush_channel();
			for event in reader.read() {
				match event {
					window::LyWindowEvent::WindowClose => should_exit = true,
					window::LyWindowEvent::MousePressed(key) => {
						info!("receiving pressed {}", key)
					}
					window::LyWindowEvent::MouseReleased(key) => {
						info!("receiving released {}", key)
					}
					_ => (),
				}
			}
			if should_exit {
				info!("exiting listener thread!");
				break;
			}
		}
	});

	let event_handler = window::get_sync_forwarding_event_loop(channel);
	window.run(event_handler);
}
