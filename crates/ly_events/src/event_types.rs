//! Provides event types to be used with the LY engine

use ly_input::{Key, MouseButton};

#[derive(Debug)]
/// Buttons, mouse and keyboard
pub enum ButtonEvent
{
	MouseScroll(f64, f64),
	MousePressed(MouseButton),
	MouseReleased(MouseButton),
	KeyPressed(Key),
	KeyReleased(Key),
}

#[derive(Debug)]
/// Event related to moving mouse
pub enum MouseEvent
{
	/// Event reporting the pixel coordinates the cursor has moved to
	///
	/// Well suited for cursor-like behaviour
	/// Should not be used to implement non-cursor functionality,
	/// use [MouseMove] instead
	CursorMove(f64, f64),

	/// Event reporting the delta the device has moved
	///
	/// Raw, unfiltered, data. Well suited for behaviours such as
	/// cameracontrol.
	MouseMove(f64, f64),
}

#[derive(Debug)]
pub enum WindowEvent
{
	WindowResized(usize, usize),
	WindowClose,
}
