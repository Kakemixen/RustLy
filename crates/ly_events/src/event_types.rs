//! Provides event types to be used with the LY engine

#[derive(Debug)]
pub enum InputEvent
{
	MouseMove(i32, i32),
	MousePressed(i32),
	MouseReleased(i32),
	KeyPressed(i32),
	KeyReleased(i32),
}

#[derive(Debug)]
pub enum WindowEvent
{
	WindowResized(usize, usize),
	WindowClose,
}
