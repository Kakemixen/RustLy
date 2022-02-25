use core::panic;

use ly_events::types::{InputEvent, WindowEvent};
use ly_input::{Key, MouseButton as LyMouseBtn};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};

fn convert_key_state(key: Key, state: ElementState) -> InputEvent
{
	use ElementState::*;
	match state {
		Pressed => InputEvent::KeyPressed(key),
		Released => InputEvent::KeyReleased(key),
	}
}

pub(crate) fn convert_keyboard_input(e: KeyboardInput) -> InputEvent
{
	let state = e.state;
	if let Some(key) = e.virtual_keycode {
		match key {
			VirtualKeyCode::Key1 => convert_key_state(Key::Key1, state),
			VirtualKeyCode::Key2 => convert_key_state(Key::Key2, state),
			VirtualKeyCode::Key3 => convert_key_state(Key::Key3, state),
			VirtualKeyCode::Key4 => convert_key_state(Key::Key4, state),
			VirtualKeyCode::Key5 => convert_key_state(Key::Key5, state),
			VirtualKeyCode::Key6 => convert_key_state(Key::Key6, state),
			VirtualKeyCode::Key7 => convert_key_state(Key::Key7, state),
			VirtualKeyCode::Key8 => convert_key_state(Key::Key8, state),
			VirtualKeyCode::Key9 => convert_key_state(Key::Key9, state),
			VirtualKeyCode::Key0 => convert_key_state(Key::Key0, state),
			VirtualKeyCode::A => convert_key_state(Key::A, state),
			VirtualKeyCode::B => convert_key_state(Key::B, state),
			VirtualKeyCode::C => convert_key_state(Key::C, state),
			VirtualKeyCode::D => convert_key_state(Key::D, state),
			VirtualKeyCode::E => convert_key_state(Key::E, state),
			VirtualKeyCode::F => convert_key_state(Key::F, state),
			VirtualKeyCode::G => convert_key_state(Key::G, state),
			VirtualKeyCode::H => convert_key_state(Key::H, state),
			VirtualKeyCode::I => convert_key_state(Key::I, state),
			VirtualKeyCode::J => convert_key_state(Key::J, state),
			VirtualKeyCode::K => convert_key_state(Key::K, state),
			VirtualKeyCode::L => convert_key_state(Key::L, state),
			VirtualKeyCode::M => convert_key_state(Key::M, state),
			VirtualKeyCode::N => convert_key_state(Key::N, state),
			VirtualKeyCode::O => convert_key_state(Key::O, state),
			VirtualKeyCode::P => convert_key_state(Key::P, state),
			VirtualKeyCode::Q => convert_key_state(Key::Q, state),
			VirtualKeyCode::R => convert_key_state(Key::R, state),
			VirtualKeyCode::S => convert_key_state(Key::S, state),
			VirtualKeyCode::T => convert_key_state(Key::T, state),
			VirtualKeyCode::U => convert_key_state(Key::U, state),
			VirtualKeyCode::V => convert_key_state(Key::V, state),
			VirtualKeyCode::W => convert_key_state(Key::W, state),
			VirtualKeyCode::X => convert_key_state(Key::X, state),
			VirtualKeyCode::Y => convert_key_state(Key::Y, state),
			VirtualKeyCode::Z => convert_key_state(Key::Z, state),
			VirtualKeyCode::Escape => convert_key_state(Key::Escape, state),
			VirtualKeyCode::F1 => convert_key_state(Key::F1, state),
			VirtualKeyCode::F2 => convert_key_state(Key::F2, state),
			VirtualKeyCode::F3 => convert_key_state(Key::F3, state),
			VirtualKeyCode::F4 => convert_key_state(Key::F4, state),
			VirtualKeyCode::F5 => convert_key_state(Key::F5, state),
			VirtualKeyCode::F6 => convert_key_state(Key::F6, state),
			VirtualKeyCode::F7 => convert_key_state(Key::F7, state),
			VirtualKeyCode::F8 => convert_key_state(Key::F8, state),
			VirtualKeyCode::F9 => convert_key_state(Key::F9, state),
			VirtualKeyCode::F10 => convert_key_state(Key::F10, state),
			VirtualKeyCode::F11 => convert_key_state(Key::F11, state),
			VirtualKeyCode::F12 => convert_key_state(Key::F12, state),
			VirtualKeyCode::F13 => convert_key_state(Key::F13, state),
			VirtualKeyCode::F14 => convert_key_state(Key::F14, state),
			VirtualKeyCode::F15 => convert_key_state(Key::F15, state),
			VirtualKeyCode::F16 => convert_key_state(Key::F16, state),
			VirtualKeyCode::F17 => convert_key_state(Key::F17, state),
			VirtualKeyCode::F18 => convert_key_state(Key::F18, state),
			VirtualKeyCode::F19 => convert_key_state(Key::F19, state),
			VirtualKeyCode::F20 => convert_key_state(Key::F20, state),
			VirtualKeyCode::F21 => convert_key_state(Key::F21, state),
			VirtualKeyCode::F22 => convert_key_state(Key::F22, state),
			VirtualKeyCode::F23 => convert_key_state(Key::F23, state),
			VirtualKeyCode::F24 => convert_key_state(Key::F24, state),
			VirtualKeyCode::Snapshot => convert_key_state(Key::PrintScreen, state),
			VirtualKeyCode::Scroll => convert_key_state(Key::ScrollLock, state),
			VirtualKeyCode::Pause => convert_key_state(Key::Pause, state),
			VirtualKeyCode::Insert => convert_key_state(Key::Insert, state),
			VirtualKeyCode::Home => convert_key_state(Key::Home, state),
			VirtualKeyCode::Delete => convert_key_state(Key::Delete, state),
			VirtualKeyCode::End => convert_key_state(Key::End, state),
			VirtualKeyCode::PageDown => convert_key_state(Key::PageDown, state),
			VirtualKeyCode::PageUp => convert_key_state(Key::PageUp, state),
			VirtualKeyCode::Left => convert_key_state(Key::Left, state),
			VirtualKeyCode::Up => convert_key_state(Key::Up, state),
			VirtualKeyCode::Right => convert_key_state(Key::Right, state),
			VirtualKeyCode::Down => convert_key_state(Key::Down, state),
			VirtualKeyCode::Back => convert_key_state(Key::Backspace, state),
			VirtualKeyCode::Return => convert_key_state(Key::Return, state),
			VirtualKeyCode::Space => convert_key_state(Key::Space, state),
			VirtualKeyCode::Compose => convert_key_state(Key::Compose, state),
			VirtualKeyCode::Caret => convert_key_state(Key::Caret, state),
			VirtualKeyCode::Numlock => convert_key_state(Key::Numlock, state),
			VirtualKeyCode::Numpad0 => convert_key_state(Key::Numpad0, state),
			VirtualKeyCode::Numpad1 => convert_key_state(Key::Numpad1, state),
			VirtualKeyCode::Numpad2 => convert_key_state(Key::Numpad2, state),
			VirtualKeyCode::Numpad3 => convert_key_state(Key::Numpad3, state),
			VirtualKeyCode::Numpad4 => convert_key_state(Key::Numpad4, state),
			VirtualKeyCode::Numpad5 => convert_key_state(Key::Numpad5, state),
			VirtualKeyCode::Numpad6 => convert_key_state(Key::Numpad6, state),
			VirtualKeyCode::Numpad7 => convert_key_state(Key::Numpad7, state),
			VirtualKeyCode::Numpad8 => convert_key_state(Key::Numpad8, state),
			VirtualKeyCode::Numpad9 => convert_key_state(Key::Numpad9, state),
			VirtualKeyCode::AbntC1 => convert_key_state(Key::AbntC1, state),
			VirtualKeyCode::AbntC2 => convert_key_state(Key::AbntC2, state),
			VirtualKeyCode::NumpadAdd => convert_key_state(Key::NumpadAdd, state),
			VirtualKeyCode::Apostrophe => convert_key_state(Key::Apostrophe, state),
			VirtualKeyCode::Apps => convert_key_state(Key::Apps, state),
			VirtualKeyCode::Asterisk => convert_key_state(Key::Asterisk, state),
			VirtualKeyCode::Plus => convert_key_state(Key::Plus, state),
			VirtualKeyCode::At => convert_key_state(Key::At, state),
			VirtualKeyCode::Ax => convert_key_state(Key::Ax, state),
			VirtualKeyCode::Backslash => convert_key_state(Key::Backslash, state),
			VirtualKeyCode::Calculator => convert_key_state(Key::Calculator, state),
			VirtualKeyCode::Capital => convert_key_state(Key::Capital, state),
			VirtualKeyCode::Colon => convert_key_state(Key::Colon, state),
			VirtualKeyCode::Comma => convert_key_state(Key::Comma, state),
			VirtualKeyCode::Convert => convert_key_state(Key::Convert, state),
			VirtualKeyCode::NumpadDecimal => convert_key_state(Key::NumpadDecimal, state),
			VirtualKeyCode::NumpadDivide => convert_key_state(Key::NumpadDivide, state),
			VirtualKeyCode::Equals => convert_key_state(Key::Equals, state),
			VirtualKeyCode::Grave => convert_key_state(Key::Grave, state),
			VirtualKeyCode::Kana => convert_key_state(Key::Kana, state),
			VirtualKeyCode::Kanji => convert_key_state(Key::Kanji, state),
			VirtualKeyCode::LAlt => convert_key_state(Key::LAlt, state),
			VirtualKeyCode::LBracket => convert_key_state(Key::LBracket, state),
			VirtualKeyCode::LControl => convert_key_state(Key::LControl, state),
			VirtualKeyCode::LShift => convert_key_state(Key::LShift, state),
			VirtualKeyCode::LWin => convert_key_state(Key::LWin, state),
			VirtualKeyCode::Mail => convert_key_state(Key::Mail, state),
			VirtualKeyCode::MediaSelect => convert_key_state(Key::MediaSelect, state),
			VirtualKeyCode::MediaStop => convert_key_state(Key::MediaStop, state),
			VirtualKeyCode::Minus => convert_key_state(Key::Minus, state),
			VirtualKeyCode::NumpadMultiply => convert_key_state(Key::NumpadMultiply, state),
			VirtualKeyCode::Mute => convert_key_state(Key::Mute, state),
			VirtualKeyCode::MyComputer => convert_key_state(Key::MyComputer, state),
			VirtualKeyCode::NavigateForward => convert_key_state(Key::NavigateForward, state),
			VirtualKeyCode::NavigateBackward => convert_key_state(Key::NavigateBackward, state),
			VirtualKeyCode::NextTrack => convert_key_state(Key::NextTrack, state),
			VirtualKeyCode::NoConvert => convert_key_state(Key::NoConvert, state),
			VirtualKeyCode::NumpadComma => convert_key_state(Key::NumpadComma, state),
			VirtualKeyCode::NumpadEnter => convert_key_state(Key::NumpadEnter, state),
			VirtualKeyCode::NumpadEquals => convert_key_state(Key::NumpadEquals, state),
			VirtualKeyCode::OEM102 => convert_key_state(Key::Oem102, state),
			VirtualKeyCode::Period => convert_key_state(Key::Period, state),
			VirtualKeyCode::PlayPause => convert_key_state(Key::PlayPause, state),
			VirtualKeyCode::Power => convert_key_state(Key::Power, state),
			VirtualKeyCode::PrevTrack => convert_key_state(Key::PrevTrack, state),
			VirtualKeyCode::RAlt => convert_key_state(Key::RAlt, state),
			VirtualKeyCode::RBracket => convert_key_state(Key::RBracket, state),
			VirtualKeyCode::RControl => convert_key_state(Key::RControl, state),
			VirtualKeyCode::RShift => convert_key_state(Key::RShift, state),
			VirtualKeyCode::RWin => convert_key_state(Key::RWin, state),
			VirtualKeyCode::Semicolon => convert_key_state(Key::Semicolon, state),
			VirtualKeyCode::Slash => convert_key_state(Key::Slash, state),
			VirtualKeyCode::Sleep => convert_key_state(Key::Sleep, state),
			VirtualKeyCode::Stop => convert_key_state(Key::Stop, state),
			VirtualKeyCode::NumpadSubtract => convert_key_state(Key::NumpadSubtract, state),
			VirtualKeyCode::Sysrq => convert_key_state(Key::Sysrq, state),
			VirtualKeyCode::Tab => convert_key_state(Key::Tab, state),
			VirtualKeyCode::Underline => convert_key_state(Key::Underline, state),
			VirtualKeyCode::Unlabeled => convert_key_state(Key::Unlabeled, state),
			VirtualKeyCode::VolumeDown => convert_key_state(Key::VolumeDown, state),
			VirtualKeyCode::VolumeUp => convert_key_state(Key::VolumeUp, state),
			VirtualKeyCode::Wake => convert_key_state(Key::Wake, state),
			VirtualKeyCode::WebBack => convert_key_state(Key::WebBack, state),
			VirtualKeyCode::WebFavorites => convert_key_state(Key::WebFavorites, state),
			VirtualKeyCode::WebForward => convert_key_state(Key::WebForward, state),
			VirtualKeyCode::WebHome => convert_key_state(Key::WebHome, state),
			VirtualKeyCode::WebRefresh => convert_key_state(Key::WebRefresh, state),
			VirtualKeyCode::WebSearch => convert_key_state(Key::WebSearch, state),
			VirtualKeyCode::WebStop => convert_key_state(Key::WebStop, state),
			VirtualKeyCode::Yen => convert_key_state(Key::Yen, state),
			VirtualKeyCode::Copy => convert_key_state(Key::Copy, state),
			VirtualKeyCode::Paste => convert_key_state(Key::Paste, state),
			VirtualKeyCode::Cut => convert_key_state(Key::Cut, state),
		}
	}
	else {
		// Win buttons on linux i3
		if e.scancode == 125 {
			return convert_key_state(Key::LWin, state);
		}
		if e.scancode == 126 {
			return convert_key_state(Key::RWin, state);
		}

		convert_key_state(Key::Other(e.scancode), state)
	}
}

fn convert_mousebtn_state(key: LyMouseBtn, state: ElementState) -> InputEvent
{
	use ElementState::*;
	match state {
		Pressed => InputEvent::MousePressed(key),
		Released => InputEvent::MouseReleased(key),
	}
}

pub(crate) fn convert_mouse_button(b: MouseButton, s: ElementState) -> InputEvent
{
	match b {
		MouseButton::Left => convert_mousebtn_state(LyMouseBtn::Left, s),
		MouseButton::Right => convert_mousebtn_state(LyMouseBtn::Right, s),
		MouseButton::Middle => convert_mousebtn_state(LyMouseBtn::Middle, s),
		MouseButton::Other(o) => convert_mousebtn_state(LyMouseBtn::Other(o), s),
	}
}

pub(crate) fn convert_mouse_move(p: winit::dpi::PhysicalPosition<f64>) -> InputEvent
{
	InputEvent::MouseMove(p.x, p.y)
}
