use std::any::TypeId;
use std::collections::HashMap;
use component::Component;
use crate::entity::{Entity, EntityId};

pub struct World {
    next_entity_id: u32,
    pub entities: HashMap<EntityId, Entity>,
    pub components: HashMap<TypeId, HashMap<EntityId, dyn Component>>
}

impl World {
    pub fn spawn(&mut self, name: Option<String>, parent: Option<EntityId>) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
    
        self.entities.insert(
            id,
            Entity {
                id,
                name,
                parent,
                children: Vec::new(),
            },
        );
    
        if let Some(parent_id) = parent {
            if let Some(parent_entity) = self.entities.get_mut(&parent_id) {
                parent_entity.children.push(id);
            }
        }
    
        id
    }
}