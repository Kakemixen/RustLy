#![feature(trait_alias)]

mod winit_converters;
use winit_converters as converters;

use ly_app::{App, AppRunner};
use ly_events::channel::SyncEventChannel;
use ly_events::types::{ButtonEvent, MouseEvent, WindowEvent};
use ly_log::core_prelude::*;
use winit::event;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::Window;

// If moving back to event_loop.run(), this is correct EventHandler
//pub trait EventHandler =
//	'static + FnMut(event::Event<'_, ()>, &EventLoopWindowTarget<()>, &mut
// ControlFlow);

pub trait EventHandler = FnMut(event::Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow);

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
	pub fn run<E>(mut self, event_handler: E, end_handler: Box<dyn FnOnce()>)
	where
		E: EventHandler,
	{
		self.event_loop.run_return(event_handler);
		end_handler();
	}

	/// Consumes window to generate a runner for [`App`](ly_app::App)
	pub fn get_app_runner(self) -> Box<AppRunner>
	{
		let closure = move |app: App| {
			let event_handler = get_sync_forwarding_event_loop(app);
			self.run(event_handler, Box::new(|| ()))
		};
		Box::new(closure)
	}
}

pub fn get_empty_event_loop() -> Box<dyn EventHandler>
{
	Box::new(move |event, _, control_flow: &mut ControlFlow| {
		if let event::Event::WindowEvent {
			event: event::WindowEvent::CloseRequested,
			..
		} = event
		{
			println!("The close button was pressed; stopping");
			*control_flow = ControlFlow::Exit;
		}
	})
}

pub fn get_sync_forwarding_event_loop<'a>(mut app: App) -> Box<dyn EventHandler + 'a>
{
	let writer_window = app
		.world
		.get_resource::<SyncEventChannel<WindowEvent>>()
		.unwrap()
		.get_writer();
	let writer_button = app
		.world
		.get_resource::<SyncEventChannel<ButtonEvent>>()
		.unwrap()
		.get_writer();
	let writer_mouse = app
		.world
		.get_resource::<SyncEventChannel<MouseEvent>>()
		.unwrap()
		.get_writer();

	Box::new(
		move |event, _, control_flow: &mut ControlFlow| match event {
			event::Event::WindowEvent {
				event,
				window_id: _winit_window_id,
				..
			} => match event {
				event::WindowEvent::CloseRequested => {
					core_info!("closing window");
					writer_window.send(WindowEvent::WindowClose);
					*control_flow = ControlFlow::Exit;
				}
				event::WindowEvent::MouseInput { button, state, .. } => {
					writer_button.send(converters::convert_mouse_button(button, state));
				}
				event::WindowEvent::CursorMoved { position, .. } => {
					writer_mouse.send(converters::convert_cursor_move(position));
				}
				event::WindowEvent::KeyboardInput { input, .. } => {
					writer_button.send(converters::convert_keyboard_input(input));
				}
				event::WindowEvent::MouseWheel { delta, .. } => {
					writer_button.send(converters::convert_mouse_scroll(delta));
				}
				_ => (),
			},
			event::Event::DeviceEvent {
				event: event::DeviceEvent::MouseMotion { delta },
				device_id: _winit_device_id,
			} => {
				writer_mouse.send(converters::convert_mouse_move(delta));
			}
			event::Event::MainEventsCleared => {}
			_ => app.update(),
		},
	)
}
