pub mod host;
pub mod polar;
pub mod query;

pub use polar_core::polar::Polar as PolarCore;

pub struct Oso(polar::Polar);

impl std::ops::Deref for Oso {
    type Target = polar::Polar;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Oso {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Oso {
    pub fn new() -> Self {
        Self(polar::Polar::new())
    }
}
