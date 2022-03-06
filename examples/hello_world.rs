use std::sync::atomic::{AtomicUsize, Ordering};

use ly_app::{AppInfo, AppState, World};
use ly_events::channel::wait_any_new_timeout;
use rustly::app::App;
use rustly::events::channel::EventWaiter;
use rustly::events::channel::SyncEventChannel;
use rustly::events::types::{ButtonEvent, MouseEvent, WindowEvent};
use rustly::log::*;
use rustly::window;
use std::thread;

#[derive(Debug)]
struct MyEvent {}

fn main()
{
	let mut app = App::new();
	let window = window::create_window().unwrap();

	app.world
		.create_resource::<SyncEventChannel<ButtonEvent>>()
		.unwrap();
	app.world
		.create_resource::<SyncEventChannel<MouseEvent>>()
		.unwrap();
	app.world.create_resource::<AtomicUsize>().unwrap();

	let runner = window.get_app_runner();
	app.add_process(thing_i_want_to_do);
	app.add_system(basic_system);
	app.set_runner(runner);
	app.run();
}

fn basic_system(world: &World)
{
	if let Ok(count) = world.get_resource::<AtomicUsize>() {
		count.fetch_add(1, Ordering::Relaxed);
	}
}

fn thing_i_want_to_do(world: &World)
{
	let reader_m = world
		.get_resource::<SyncEventChannel<MouseEvent>>()
		.unwrap()
		.get_reader();
	let reader_b = world
		.get_resource::<SyncEventChannel<ButtonEvent>>()
		.unwrap()
		.get_reader();

	let arr: [&dyn EventWaiter; 2] = [&reader_b, &reader_m];

	loop {
		// need this for some reason, or it will drop events
		// TODO why?
		// thread::sleep(Duration::from_millis(1));
		// wait_new solves it here, but not root cause
		//reader_b.wait_new();

		debug!("waiting...");
		wait_any_new_timeout(&arr, 500);
		if let AppState::Stopped = world.get_resource::<AppInfo>().unwrap().state() {
			info!("Application quit, breaking read loop!");
			break;
		}
		//wait_any_new(&[&reader_b as &dyn EventWaiter]);
		debug!("got new...");

		reader_b.flush_channel();
		for event in reader_b.read() {
			if let ButtonEvent::MousePressed(ly_input::MouseButton::Left) = event {
				let count = world.get_resource::<AtomicUsize>().unwrap();
				debug!("number of updates {:?}", count);
			}
			info!("recieved {:?}", event);
		}

		reader_m.flush_channel();
		for event in reader_m.read() {
			info!("recieved {:?}", event);
		}

		if !reader_b.channel_has_writers() {
			warning!("button no longer has readers");
			break;
		}
		if !reader_m.channel_has_writers() {
			warning!("mouse no longer has readers");
			break;
		}
	}
}
