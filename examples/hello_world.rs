use rustly::events::types::{ButtonEvent, MouseEvent, WindowEvent};
use rustly::log::*;
use rustly::{events, window};
use std::sync::Arc;
use std::thread;
//use std::time::Duration;

use rustly::events::channel::{wait_any_new, EventWaiter};

fn main()
{
	log_init();
	let window = window::create_window();

	let channel_button = Arc::new(events::channel::SyncEventChannel::<ButtonEvent>::new());
	let channel_mouse = Arc::new(events::channel::SyncEventChannel::<MouseEvent>::new());
	let channel_window = Arc::new(events::channel::SyncEventChannel::<WindowEvent>::new());

	//let cw = Arc::clone(&channel_window);
	let cb = Arc::clone(&channel_button);
	let cm = Arc::clone(&channel_mouse);
	let handle = thread::Builder::new()
		.name("Logging thread".to_string())
		.spawn(move || {
			//let reader_w = cw.get_reader();
			let reader_b = cb.get_reader();
			let reader_m = cm.get_reader();
			let arr: [&dyn EventWaiter; 2] = [&reader_b, &reader_m];

			loop {
				// need this for some reason, or it will drop events
				// TODO why?
				// thread::sleep(Duration::from_millis(1));
				// wait_new solves it here, but not root cause
				//reader_b.wait_new();

				wait_any_new(&arr);
				//wait_any_new(&[&reader_b as &dyn EventWaiter]);

				reader_b.flush_channel();
				for event in reader_b.read() {
					match event {
						e => {
							info!("recieved {:?}", e);
						}
					}
				}

				reader_m.flush_channel();
				for event in reader_m.read() {
					match event {
						e => {
							info!("recieved {:?}", e);
						}
					}
				}

				if !reader_b.channel_has_writers() {
					break;
				}
				if !reader_m.channel_has_writers() {
					break;
				}
			}
		})
		.unwrap();

	let writer_button = channel_button.get_writer();
	let writer_mouse = channel_mouse.get_writer();
	let writer_window = channel_window.get_writer();

	let event_handler =
		window::get_sync_forwarding_event_loop(writer_window, writer_button, writer_mouse);
	window.run(
		event_handler,
		Box::new(move || {
			let _x = handle.join();
		}),
	);
}
