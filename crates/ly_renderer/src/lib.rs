mod ash_window;

use ly_log::core_prelude::*;

use ash::{vk, Entry};
use raw_window_handle::HasRawWindowHandle;
use std::error::Error;
//use scopeguard;

use std::ffi::CStr;

pub struct LyRenderer {}

impl LyRenderer
{
	pub fn new(window: &dyn HasRawWindowHandle) -> Result<Self, Box<dyn Error>>
	{
		let entry = Entry::linked();
		let extensions = get_required_instance_extensions(window).expect("get extensions fail!");
		check_required_instance_extensions(&entry, &extensions)?;
		Ok(LyRenderer {})
	}
}

fn get_required_instance_extensions(
	window: &dyn HasRawWindowHandle,
) -> Result<Vec<&'static CStr>, Box<dyn Error>>
{
	let mut instance_extensions = match ash_window::enumerate_required_extensions(window) {
		Ok(extensions) => extensions,
		Err(_) => {
			return Err("failed to enumerate required instance extensions".into());
		}
	};

	instance_extensions.push(ash::extensions::ext::DebugUtils::name());
	Ok(instance_extensions)
}

fn check_required_instance_extensions<'a>(
	entry: &ash::Entry,
	required_instance_extensions: &Vec<&'a std::ffi::CStr>,
) -> Result<(), String>
{
	core_info!(
		"checking required instance extensions: {:?}",
		required_instance_extensions
	);

	let supported_instance_extensions = match entry.enumerate_instance_extension_properties(None) {
		Ok(props) => props,
		Err(_) => {
			return Err(String::from(
				"failed to enumerate instance extension properies",
			));
		}
	};

	let mut supported_instance_extensions_set = std::collections::HashSet::new();
	for vk::ExtensionProperties { extension_name, .. } in &supported_instance_extensions {
		supported_instance_extensions_set
			.insert(unsafe { std::ffi::CStr::from_ptr(extension_name.as_ptr()) });
	}

	for &extension_name in required_instance_extensions {
		if !supported_instance_extensions_set.contains(extension_name) {
			return Err(format!(
				"instance extension {:?} is not supported",
				extension_name
			));
		}
	}

	core_debug!("all extensions are supported",);

	Ok(())
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
