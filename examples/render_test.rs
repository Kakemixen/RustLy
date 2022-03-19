use rustly::app::App;
use rustly::events::channel::SyncEventChannel;
use rustly::events::types::*;
use rustly::log::*;
use rustly::renderer;
use rustly::window;

pub fn main()
{
	let app = App::new();
	let window = window::create_window().unwrap();

	app.world
		.create_resource::<SyncEventChannel<ButtonEvent>>()
		.unwrap();
	app.world
		.create_resource::<SyncEventChannel<MouseEvent>>()
		.unwrap();
	app.world
		.create_resource::<SyncEventChannel<WindowEvent>>()
		.unwrap();

	match renderer::LyRenderer::new(window.get_handle()) {
		Ok(_) => info!("All is good!"),
		Err(e) => {
			error!("error: {}", e)
		}
	}
}
