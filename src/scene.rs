use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result};
use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::entity::EntityId;
use asset::AssetId;
use component::{Component, IntoPropertyType, PropertyDescriptor, transform::TransformComponent};
use component_derive::*;

use crate::{
    camera::{Camera, CameraUniform},
    renderer::GpuContext,
    texture::Texture,
};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MeshHandle(usize);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaterialHandle(usize);

#[derive(Clone, Copy, Debug, Component)]
pub struct MeshRendererComponent {
    pub mesh: AssetId,
    pub material: AssetId,
}

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct CameraComponent {
    #[property(hidden)]
    pub camera: Camera,
}

impl CameraComponent {
    pub fn new(camera: Camera) -> Self {
        Self { camera }
    }

    pub fn uniform(&self) -> CameraUniform {
        CameraUniform::from_camera(&self.camera)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn from_transform(transform: &TransformComponent) -> Self {
        Self {
            model: transform.matrix().into(),
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(ctx: &GpuContext, label: &str, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{label} Vertex Buffer")),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{label} Index Buffer")),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}

pub struct Material {
    #[allow(dead_code)]
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn from_texture_bytes(
        ctx: &GpuContext,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let texture = Texture::from_bytes(&ctx.device, &ctx.queue, bytes, label)?;
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some(label),
        });

        Ok(Self {
            texture,
            bind_group,
        })
    }
}

pub struct RenderBatch {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
}

#[derive(Default)]
pub struct Scene {
    pub transforms: HashMap<EntityId, TransformComponent>,
    pub mesh_renderers: HashMap<EntityId, MeshRendererComponent>,
    pub cameras: HashMap<EntityId, CameraComponent>,
    pub active_camera: Option<EntityId>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub render_batches: Vec<RenderBatch>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    // pub fn default_instanced(
    //     ctx: &GpuContext,
    //     texture_bind_group_layout: &wgpu::BindGroupLayout,
    //     aspect: f32,
    // ) -> Result<Self> {
    //     let mut scene = Self::new();
    //     let mesh = scene.add_mesh(Mesh::new(
    //         ctx,
    //         "Pentagon",
    //         PENTAGON_VERTICES,
    //         PENTAGON_INDICES,
    //     ));
    //     let material = scene.add_material(Material::from_texture_bytes(
    //         ctx,
    //         texture_bind_group_layout,
    //         include_bytes!("happy-tree.png"),
    //         "happy-tree.png",
    //     )?);

    //     let camera = Camera::new((0.0, 5.0, 10.0).into(), (0.0, 0.0, 0.0).into(), aspect);
    //     let camera_entity = scene.spawn(Some("Main Camera".to_owned()), None);
    //     scene.add_camera(camera_entity, CameraComponent::new(camera));
    //     scene.active_camera = Some(camera_entity);

    //     for z in 0..NUM_INSTANCES_PER_ROW {
    //         for x in 0..NUM_INSTANCES_PER_ROW {
    //             let position = cgmath::Vector3 {
    //                 x: x as f32,
    //                 y: 0.0,
    //                 z: z as f32,
    //             } - INSTANCE_DISPLACEMENT;

    //             let rotation = if position.is_zero() {
    //                 cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
    //             } else {
    //                 cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
    //             };

    // let entity = scene.spawn(Some(format!("Pentagon {x},{z}")), None);
    // scene.set_transform(
    //     entity,
    //     TransformComponent::from_translation_rotation(position, rotation),
    // );
    // scene.add_mesh_renderer(entity, MeshRendererComponent { mesh, material });
    //     }
    // }

    //     scene.rebuild_render_batches(ctx);
    //     Ok(scene)
    // }

    pub fn set_transform(&mut self, entity: EntityId, transform: TransformComponent) {
        self.transforms.insert(entity, transform);
    }

    pub fn add_mesh_renderer(&mut self, entity: EntityId, component: MeshRendererComponent) {
        self.mesh_renderers.insert(entity, component);
    }

    pub fn add_camera(&mut self, entity: EntityId, component: CameraComponent) {
        self.cameras.insert(entity, component);
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshHandle {
        let handle = MeshHandle(self.meshes.len());
        self.meshes.push(mesh);
        handle
    }

    pub fn add_material(&mut self, material: Material) -> MaterialHandle {
        let handle = MaterialHandle(self.materials.len());
        self.materials.push(material);
        handle
    }

    pub fn mesh(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(handle.0)
    }

    pub fn material(&self, handle: MaterialHandle) -> Option<&Material> {
        self.materials.get(handle.0)
    }

    pub fn active_camera_uniform(&self) -> Option<CameraUniform> {
        let camera_id = self.active_camera?;
        let camera = self.cameras.get(&camera_id)?;
        Some(CameraUniform::from_camera(&camera.camera))
    }

    pub fn set_active_camera_aspect(&mut self, aspect: f32) {
        if let Some(camera_id) = self.active_camera {
            if let Some(camera) = self.cameras.get_mut(&camera_id) {
                camera.camera.set_aspect(aspect);
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.active_camera
            .context("scene has no active camera")
            .and_then(|camera_id| {
                self.cameras
                    .contains_key(&camera_id)
                    .then_some(())
                    .context("active camera entity has no CameraComponent")
            })
    }
}

const PENTAGON_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    },
];

const PENTAGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, 0];
