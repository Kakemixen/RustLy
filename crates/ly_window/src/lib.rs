#![feature(trait_alias)]

use ly_events as events;
use winit::event;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::Window;

pub enum LyWindowEvent
{
	MouseMove(i32, i32),
	MousePressed(i32),
	MouseReleased(i32),
	KeyPressed(i32),
	KeyReleased(i32),
	WindowClose(),
}

pub trait EventHandler =
	'static + FnMut(event::Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow);

pub struct LyWindow
{
	event_loop: EventLoop<()>,
	window: Window,
}

pub fn create_window() -> LyWindow
{
	// window
	let event_loop = EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_title("Initial Window")
		//.with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
		//.with_min_inner_size(winit::dpi::PhysicalSize::new(100.0, 100.0))
		.build(&event_loop)
		.unwrap();

	LyWindow { event_loop, window }
}

impl LyWindow
{
	pub fn run<E>(self, event_handler: E)
	where
		E: EventHandler,
	{
		self.event_loop.run(event_handler);
	}
}

pub fn get_empty_event_loop() -> Box<dyn EventHandler>
{
	Box::new(
		move |event, _, control_flow: &mut ControlFlow| match event {
			event::Event::WindowEvent {
				event: event::WindowEvent::CloseRequested,
				..
			} => {
				println!("The close button was pressed; stopping");
				*control_flow = ControlFlow::Exit;
			}
			_ => (),
		},
	)
}

#[cfg(test)]
mod tests
{
	#[test]
	fn it_works()
	{
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}
