use ly_app::World;
use ly_events::channel::wait_any_new;
use rustly::app::App;
use rustly::events::channel::EventWaiter;
use rustly::events::channel::SyncEventChannel;
use rustly::events::types::{ButtonEvent, MouseEvent, WindowEvent};
use rustly::log::*;
use rustly::window;

#[derive(Debug)]
struct MyEvent {}

fn main()
{
	let mut app = App::new();
	let window = window::create_window();

	let channel_button = SyncEventChannel::<ButtonEvent>::new();
	let channel_mouse = SyncEventChannel::<MouseEvent>::new();
	let channel_window = SyncEventChannel::<WindowEvent>::new();
	app.world.set_resource(channel_button).unwrap();
	app.world.set_resource(channel_mouse).unwrap();
	app.world.set_resource(channel_window).unwrap();

	let runner = window.get_app_runner();
	app.add_process(thing_i_want_to_do);
	app.set_runner(runner);
	app.run();
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
}
