mod world;

pub use world::World;

use ly_log::core_prelude::*;
use std::process::exit;

pub type AppRunner = dyn FnOnce(App) -> ();

/// The Application, should be only one
#[derive(Default)]
pub struct App
{
	world: World,
	runner: Option<Box<AppRunner>>,
}

impl App
{
	pub fn new() -> Self
	{
		log_init();
		App::default()
	}

	/// Insert a resource into the global storage.
	/// See [`World::set_resource`]
	pub fn set_resource<T>(&self, resource: T) -> Result<(), ()>
	where
		T: Send + Sync + 'static,
	{
		self.world.set_resource(resource)
	}

	/// Get a resource from the global storage.
	/// See [`World::get_resource`]
	pub fn get_resource<T>(&self) -> Result<&'static T, ()>
	where
		T: Send + Sync + 'static,
	{
		self.world.get_resource()
	}

	/// Runs the application.
	/// This will use the runner set by (`set_runner`)[App::set_runner]
	/// and hijack the running the thread.
	pub fn run(mut self) -> !
	{
		let mut exit_code = 0;
		if let Some(runner) = self.runner.take() {
			runner(self);
		}
		else {
			core_error!("No runner set, stopping!");
			exit_code = 1;
		}
		log_die("App has stopped".to_string());
		exit(exit_code);
	}

	/// Used to set a run function for this app.
	/// This fun
	pub fn set_runner(&mut self, runner: Box<AppRunner>) { self.runner = Some(runner); }
}
