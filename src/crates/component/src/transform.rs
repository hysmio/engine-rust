use crate::{Component, IntoPropertyType, PropertyDescriptor};
use cgmath::{Rotation3, Vector3};
use component_derive::*;

#[derive(Clone, Debug, Component)]
pub struct TransformComponent {
    #[property]
    pub translation: Vector3<f32>,
    #[property]
    pub rotation: cgmath::Quaternion<f32>,
    #[property]
    pub scale: cgmath::Vector3<f32>,
}

impl TransformComponent {
    pub fn identity() -> Self {
        Self {
            translation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_translation_rotation(
        translation: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
    ) -> Self {
        Self {
            translation,
            rotation,
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.translation)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self::identity()
    }
}
