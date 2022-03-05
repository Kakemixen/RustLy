mod world;

pub use world::World;

use crossbeam::thread::scope;
use ly_log::core_prelude::*;
use std::{
	process::exit,
	sync::atomic::{AtomicUsize, Ordering},
};

pub type AppRunner = dyn FnOnce(App);
//pub type AppSubProcess = dyn FnOnce(&'static World) -> () + Send;
pub type AppSubProcess = fn(&World);

/// The Application, should be only one
#[derive(Default)]
pub struct App
{
	pub world: World,
	runner: Option<Box<AppRunner>>,
	processes: Option<Vec<AppSubProcess>>,
}

impl App
{
	pub fn new() -> Self
	{
		log_init();
		App::default()
	}

	/// Runs the application.
	/// This will use the runner set by [`set_runner`](App::set_runner)
	/// and hijack the running the thread.
	pub fn run(mut self) -> !
	{
		let mut exit_code = 0;
		if let Some(runner) = self.runner.take() {
			scope(|s| {
				if let Some(procs) = self.processes.take() {
					for p in procs.into_iter() {
						let world = self.get_world_handle();
						s.spawn(move |_| p(world));
					}
				}
				runner(self);
			})
			.unwrap();
		}
		else {
			core_error!("No runner set, stopping!");
			exit_code = 1;
		}
		log_die("App has stopped".to_string());
		exit(exit_code);
	}

	/// Update tick for application
	pub fn update(&mut self)
	{
		if let Ok(count) = &mut self.world.get_resource::<AtomicUsize>() {
			count.fetch_add(1, Ordering::Relaxed);
		}
	}

	/// Used to set a run function for this app.
	/// This fun
	pub fn set_runner(&mut self, runner: Box<AppRunner>) { self.runner = Some(runner); }

	/// Add a subprocess to the app.
	/// The provided fn will we run in a separate thread and joined upon
	/// application exit
	pub fn add_process(&mut self, func: AppSubProcess)
	{
		if let Some(procs) = &mut self.processes {
			procs.push(func);
		}
		else {
			self.processes = Some(vec![func]);
		}
	}

	/// Gets a world handle to be passed to subprocess
	/// TODO: create system to pass resources to subprocess instead of the world
	fn get_world_handle(&self) -> &'static World
	{
		// SAFE: only used for scoped threads in the app
		unsafe {
			let ptr = &self.world as *const World;
			&*ptr
		}
	}
}
