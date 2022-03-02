mod world;

use world::World;

/// The Application, should be only one
pub struct App
{
	world: World,
}

impl App
{
	pub fn new() -> Self
	{
		App {
			world: World::new(),
		}
	}

	/// Insert a resource into the global storage.
	/// Returns Err if a resource of that type is set already.
	/// In that case, the resource storage is not updated,
	/// should you require mutability, use interior for now.
	pub fn set_resource<T>(&self, resource: T) -> Result<(), ()>
	where
		T: Send + Sync + 'static,
	{
		self.world.set_resource(resource)
	}

	/// Get a resource from the global storage.
	/// Returns Err if no resource of that type exists.
	pub fn get_resource<T>(&self) -> Result<&'static T, ()>
	where
		T: Send + Sync + 'static,
	{
		self.world.get_resource()
	}
}
