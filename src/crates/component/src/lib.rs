use std::any::TypeId;

pub mod transform;

pub enum PropertyType {
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    Vec2,
    Vec3,
    Vec4,
    Bool,
    String,
}

// pub enum PropertyValue {
//     F32(f32),
//     F64(f64),
//     I8(i8),
//     I16(i16),
//     I32(i32),
//     I64(i64),
//     Vec2(f32, f32),
//     Vec3(f32, f32, f32),
//     Vec4(f32, f32, f32, f32),
//     Bool(bool),
//     String(String),
// }

pub struct PropertyDescriptor {
    pub name: String,
    pub description: Option<String>,
    pub data_type: PropertyType
    // pub default: Option<PropertyValue>,
}

pub trait Component {
    fn name(&self) -> &'static str;
    fn type_id(&self) -> TypeId;

    fn properties(&self) -> Vec<&PropertyDescriptor>;
}