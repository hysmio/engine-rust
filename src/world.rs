use crate::camera::{Camera, CameraUniform};
use crate::entity::{Entity, EntityId};
use crate::scene::CameraComponent;
use component::Component;
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct ComponentStore<T> {
    sparse: Vec<Option<usize>>, // EntityId as index
    dense: Vec<EntityId>,
    data: Vec<T>,
}

impl<T: Clone> ComponentStore<T> {
    fn get(&self, entity: EntityId) -> Option<&T> {
        let slot = (*self.sparse.get(entity.0 as usize)?)?;
        Some(&self.data[slot])
    }

    fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        let slot = (*self.sparse.get(entity.0 as usize)?)?;
        self.data.get_mut(slot)
    }

    fn insert(&mut self, entity: EntityId, component: T) {
        let new_index = self.dense.len();
        if (entity.0 as usize) >= self.sparse.len() {
            self.sparse.resize(entity.0 as usize + 1, None);
        }
        
        self.sparse[entity.0 as usize] = Some(new_index);
        self.dense.push(entity);
        self.data.push(component);
    }
}

pub struct World {
    next_entity_id: u64,
    pub entities: HashMap<EntityId, Entity>,
    pub components: HashMap<TypeId, Box<dyn Any>>,

    active_camera: Option<EntityId>,
}

impl World {
    pub fn empty(aspect: f32) -> World {
        let mut world = World {
            next_entity_id: 0,
            entities: HashMap::new(),
            components: HashMap::new(),
            active_camera: None,
        };

        let camera = Camera::new((0.0, 5.0, 10.0).into(), (0.0, 0.0, 0.0).into(), aspect);
        let camera_entity = world.spawn(Some("Main Camera".to_owned()), None);
        world.add_component(camera_entity, CameraComponent::new(camera));
        world.active_camera = Some(camera_entity);

        world
    }

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

    pub fn add_component<T: Component + Sized + Default + Clone + 'static>(
        &mut self,
        entity: EntityId,
        component: T,
    ) {
        let arr = self
            .components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentStore::<T>::default()));
        arr.downcast_mut::<ComponentStore<T>>()
            .unwrap()
            .insert(entity, component);
    }

    pub fn get_component_for_entity<T: Component + Sized + Default + Clone + 'static>(
        &self,
        entity: EntityId,
    ) -> Option<&T> {
        let store = self.components.get(&TypeId::of::<T>())?; // type lookup
        let store = store.downcast_ref::<ComponentStore<T>>().unwrap(); // erase -> Column<T>
        store.get(entity) // entity lookup
    }

    pub fn get_mut_component_for_entity<T: Component + Sized + Default + Clone + 'static>(
        &mut self,
        entity: EntityId,
    ) -> Option<&mut T> {
        let store = self.components.get_mut(&TypeId::of::<T>())?; // type lookup
        let store = store.downcast_mut::<ComponentStore<T>>().unwrap(); // erase -> Column<T>
        store.get_mut(entity) // entity lookup
    }

    pub fn active_camera_uniform(&self) -> Option<CameraUniform> {
        let camera_id = self.active_camera?;
        let camera = self
            .components
            .get(&TypeId::of::<CameraComponent>())?
            .downcast_ref::<ComponentStore<CameraComponent>>()?
            .get(camera_id)?;
        Some(camera.uniform())
    }

    pub fn set_active_camera_aspect(&mut self, aspect: f32) {
        if let Some(camera_entity_id) = self.active_camera {
            if let Some(camera) =
                self.get_mut_component_for_entity::<CameraComponent>(camera_entity_id)
            {
                camera.camera.set_aspect(aspect);
            }
        }
    }
}
