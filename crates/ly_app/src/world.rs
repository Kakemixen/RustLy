use state::container::ContainerSendSync;

static CONTAINER: ContainerSendSync = ContainerSendSync::new();

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

	pub fn get_resource<T>(&self) -> Result<&'static T, ()>
	where
		T: Send + Sync + 'static,
	{
		let ret = self.resources.try_get();
		if let Some(v) = ret { Ok(v) } else { Err(()) }
	}
}
