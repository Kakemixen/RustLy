use rustly::events::types::{InputEvent, WindowEvent};
use rustly::log::*;
use rustly::{events, window};
use std::sync::Arc;
use std::thread;
//use std::time::Duration;

fn main()
{
	log_init();
	let window = window::create_window();

	let channel_window = Arc::new(events::channel::SyncEventChannel::<WindowEvent>::new());
	let channel_input = Arc::new(events::channel::SyncEventChannel::<InputEvent>::new());

	//let cw = Arc::clone(&channel_window);
	let ci = Arc::clone(&channel_input);
	let handle = thread::Builder::new()
		.name("Logging thread".to_string())
		.spawn(move || {
			//let reader_w = cw.get_reader();
			let reader_i = ci.get_reader();

			loop {
				// need this for some reason, or it will drop events
				// TODO why?
				// thread::sleep(Duration::from_millis(1));
				// wait_new solves it here, but not root cause
				reader_i.wait_new();

				reader_i.flush_channel();
				for event in reader_i.read() {
					match event {
						e => {
							info!("recieved {:?}", e);
						}
					}
				}
				if !reader_i.channel_has_writers() {
					break;
				}
			}
		})
		.unwrap();

	let writer_window = channel_window.get_writer();
	let writer_input = channel_input.get_writer();
	let event_handler = window::get_sync_forwarding_event_loop(writer_window, writer_input);
	window.run(
		event_handler,
		Box::new(move || {
			let _x = handle.join();
		}),
	);
}
