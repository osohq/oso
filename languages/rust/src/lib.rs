pub mod host;
pub mod polar;
pub mod query;

pub use polar_core::polar::Polar as PolarCore;

use host::ToPolar;

#[derive(Clone, Default)]
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

    pub fn is_allowed<Actor, Action, Resource>(
        &mut self,
        actor: Actor,
        action: Action,
        resource: Resource,
    ) -> bool
    where
        Actor: ToPolar,
        Action: ToPolar,
        Resource: ToPolar,
    {
        let args: Vec<&dyn ToPolar> = vec![&actor, &action, &resource];
        let mut query = self.0.query_rule("allow", args).unwrap();
        query.next().is_some()
    }
}
