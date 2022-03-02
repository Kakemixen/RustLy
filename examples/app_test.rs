use rustly::app;
use rustly::events::channel::SyncEventChannel;
use std::io::Read;
use std::thread;

#[derive(Debug)]
struct MyEvent {}

fn main()
{
	let app = app::App::new();
	let events = SyncEventChannel::<MyEvent>::new();
	app.set_resource(events).unwrap();
	let events = app.get_resource::<SyncEventChannel<MyEvent>>().unwrap();
	let writer = events.get_writer();

	let handle = thread::spawn(move || {
		let events = app.get_resource::<SyncEventChannel<MyEvent>>().unwrap();
		let reader = events.get_reader();
		reader.wait_new();
		reader.flush_channel();
		for event in reader.read() {
			println!("{:?}", event);
		}
	});

	std::thread::sleep_ms(50);
	writer.send(MyEvent {});
	handle.join().unwrap();
}
