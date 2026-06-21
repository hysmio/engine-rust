#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityId(pub u32);

#[derive(Debug)]
pub struct Entity {
    pub id: EntityId,
    pub name: Option<String>,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
}