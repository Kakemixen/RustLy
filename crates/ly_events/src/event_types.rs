//! Provides event types to be used with the LY engine

use ly_input::{Key, MouseButton};

#[derive(Debug)]
pub enum InputEvent
{
	MouseMove(f64, f64),
	MousePressed(MouseButton),
	MouseReleased(MouseButton),
	KeyPressed(Key),
	KeyReleased(Key),
}

#[derive(Debug)]
pub enum WindowEvent
{
	WindowResized(usize, usize),
	WindowClose,
}