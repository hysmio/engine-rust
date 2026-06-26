#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new() -> EntityId {
        EntityId(rand::random())
    }
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: EntityId,
    pub name: Option<String>,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
}
