use rustly::app;
use rustly::events::channel::SyncEventChannel;
use rustly::log::*;

#[derive(Debug)]
struct MyEvent {}

fn main()
{
	let mut app = app::App::new();
	let events = SyncEventChannel::<MyEvent>::new();
	app.set_resource(events).unwrap();
	let runner = Box::new(|_x| (info!("running the app")));
	app.set_runner(runner);
	app.run();
}
