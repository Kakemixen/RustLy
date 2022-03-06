mod world;

use parking_lot::Mutex;
pub use world::World;

use crossbeam::thread::scope;
use ly_log::core_prelude::*;
use std::process::exit;

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
	systems: Vec<AppSubProcess>,
}

/// The state of the application
#[derive(Clone, Copy)]
pub enum AppState
{
	Initialized,
	Running,
	Idle,
	Stopped,
}

/// Information about app intended to keep in storage
pub struct AppInfo
{
	state: Mutex<AppState>,
}

impl AppInfo
{
	fn new_initialized() -> Self
	{
		AppInfo {
			state: Mutex::new(AppState::Initialized),
		}
	}

	/// Get the current state of the application
	pub fn state(&self) -> AppState { *self.state.lock() }

	/// Sets new state for application
	fn set_state(&self, state: AppState) { *self.state.lock() = state; }
}

impl App
{
	pub fn new() -> Self
	{
		log_init();
		let app = App::default();
		if let Err(e) = app.world.set_resource(AppInfo::new_initialized()) {
			core_error!("Could not initialize AppInfo correctly due to {}", e)
		}
		app
	}

	/// Runs the application.
	/// This will use the runner set by [`set_runner`](App::set_runner)
	/// and hijack the running the thread.
	pub fn run(mut self) -> !
	{
		let mut exit_code = 0;
		if let Some(runner) = self.runner.take() {
			let world = self.get_world_handle();
			world
				.get_resource::<AppInfo>()
				.unwrap()
				.set_state(AppState::Running);
			scope(|s| {
				if let Some(procs) = self.processes.take() {
					for p in procs.into_iter() {
						let world = self.get_world_handle();
						s.spawn(move |_| p(world));
					}
				}
				runner(self);
				world
					.get_resource::<AppInfo>()
					.unwrap()
					.set_state(AppState::Stopped);
			})
			.unwrap();
		}
		else {
			core_error!("No runner set, stopping!");
			exit_code = 1;
		}

		// exit cleanup
		log_die("App has stopped".to_string());
		exit(exit_code);
	}

	/// Update tick for application
	pub fn update(&mut self)
	{
		for system in self.systems.iter() {
			system(&self.world);
		}
	}

	/// Used to set a run function for this app.
	pub fn set_runner(&mut self, runner: Box<AppRunner>) { self.runner = Some(runner); }

	/// Add a subprocess to the app.
	/// The provided fn will we run in a separate thread and joined upon
	/// application exit, so if the function never returns, the application
	/// hangs
	pub fn add_process(&mut self, func: AppSubProcess)
	{
		if let Some(procs) = &mut self.processes {
			procs.push(func);
		}
		else {
			self.processes = Some(vec![func]);
		}
	}

	/// Adds a system to the application.
	/// The provided fn will we run every app update in the main thread.
	pub fn add_system(&mut self, func: AppSubProcess) { self.systems.push(func); }

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
