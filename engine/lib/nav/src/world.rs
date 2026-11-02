/// The navigation state of one simulated space. It holds nothing yet.
#[derive(Debug, Default)]
pub struct World {}

impl World {
    /// Creates an empty world.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_world_is_empty() {
        assert_eq!(format!("{:?}", World::new()), "World");
    }
}
