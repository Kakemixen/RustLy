#![feature(trait_alias)]

use ly_events as events;
use ly_log::core_prelude::*;
use std::sync::Arc;
use winit::event;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::Window;

#[derive(Debug)]
pub enum LyWindowEvent
{
	MouseMove(i32, i32),
	MousePressed(i32),
	MouseReleased(i32),
	KeyPressed(i32),
	KeyReleased(i32),
	WindowClose,
}

pub trait EventHandler =
	'static + FnMut(event::Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow);

pub struct LyWindow
{
	event_loop: EventLoop<()>,
	#[allow(dead_code)]
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

pub fn get_sync_forwarding_event_loop(
	channel: Arc<events::channel::SyncEventChannel<LyWindowEvent>>,
) -> Box<dyn EventHandler>
{
	Box::new(
		move |event, _, control_flow: &mut ControlFlow| match event {
			event::Event::WindowEvent {
				event,
				window_id: _winit_window_id,
				..
			} => match event {
				event::WindowEvent::CloseRequested => {
					core_info!("closing");
					log_die("The close button was pressed; stopping".to_string());
					core_info!("closing");
					channel.send(LyWindowEvent::WindowClose);
					*control_flow = ControlFlow::Exit;
				}
				event::WindowEvent::MouseInput { button, state, .. } => match state {
					event::ElementState::Pressed => match button {
						event::MouseButton::Left => {
							channel.send(LyWindowEvent::MousePressed(0));
						}
						event::MouseButton::Right => {
							channel.send(LyWindowEvent::MousePressed(1));
						}
						_ => (),
					},
					event::ElementState::Released => match button {
						event::MouseButton::Left => {
							channel.send(LyWindowEvent::MouseReleased(0));
						}
						event::MouseButton::Right => {
							channel.send(LyWindowEvent::MouseReleased(1));
						}
						_ => (),
					},
				},
				_ => (),
			},
			event::Event::DeviceEvent {
				event,
				device_id: _winit_device_id,
			} => match event {
				_ => (),
			},
			event::Event::MainEventsCleared => {}
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
