use state::container::ContainerSendSync;

static CONTAINER: ContainerSendSync = ContainerSendSync::new();

/// The World, used to store global resources
pub struct World
{
	resources: &'static ContainerSendSync,
}

impl World
{
	pub fn new() -> Self
	{
		World {
			resources: &CONTAINER,
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
		if self.resources.set(resource) {
			Ok(())
		}
		else {
			Err(())
		}
	}

	/// Get a resource from the global storage.
	/// Returns Err if no resource of that type exists.
	pub fn get_resource<T>(&self) -> Result<&'static T, ()>
	where
		T: Send + Sync + 'static,
	{
		let ret = self.resources.try_get();
		if let Some(v) = ret { Ok(v) } else { Err(()) }
	}
}

impl Default for World
{
	fn default() -> Self { World::new() }
}
