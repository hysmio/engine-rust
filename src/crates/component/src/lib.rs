use asset::AssetId;

pub mod transform;

pub enum PropertyType {
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Vec2,
    Vec3,
    Vec4,
    Bool,
    String,
    Quat,
    AssetRef,
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
    pub data_type: PropertyType, // pub default: Option<PropertyValue>,
}

pub trait IntoPropertyType {
    const PROPERTY_TYPE: PropertyType;
}

impl IntoPropertyType for f32 {
    const PROPERTY_TYPE: PropertyType = PropertyType::F32;
}

impl IntoPropertyType for f64 {
    const PROPERTY_TYPE: PropertyType = PropertyType::F64;
}

impl IntoPropertyType for String {
    const PROPERTY_TYPE: PropertyType = PropertyType::String;
}

impl IntoPropertyType for u8 {
    const PROPERTY_TYPE: PropertyType = PropertyType::U8;
}

impl IntoPropertyType for u16 {
    const PROPERTY_TYPE: PropertyType = PropertyType::U16;
}

impl IntoPropertyType for u32 {
    const PROPERTY_TYPE: PropertyType = PropertyType::U32;
}

impl IntoPropertyType for u64 {
    const PROPERTY_TYPE: PropertyType = PropertyType::U64;
}

impl IntoPropertyType for i8 {
    const PROPERTY_TYPE: PropertyType = PropertyType::I8;
}

impl IntoPropertyType for i16 {
    const PROPERTY_TYPE: PropertyType = PropertyType::I16;
}

impl IntoPropertyType for i32 {
    const PROPERTY_TYPE: PropertyType = PropertyType::I32;
}

impl IntoPropertyType for i64 {
    const PROPERTY_TYPE: PropertyType = PropertyType::I64;
}

impl IntoPropertyType for cgmath::Vector3<f32> {
    const PROPERTY_TYPE: PropertyType = PropertyType::Vec3;
}

impl IntoPropertyType for cgmath::Vector2<f32> {
    const PROPERTY_TYPE: PropertyType = PropertyType::Vec2;
}

impl IntoPropertyType for cgmath::Vector4<f32> {
    const PROPERTY_TYPE: PropertyType = PropertyType::Vec4;
}

impl IntoPropertyType for cgmath::Quaternion<f32> {
    const PROPERTY_TYPE: PropertyType = PropertyType::Quat;
}

impl IntoPropertyType for bool {
    const PROPERTY_TYPE: PropertyType = PropertyType::Bool;
}

impl IntoPropertyType for AssetId {
    const PROPERTY_TYPE: PropertyType = PropertyType::AssetRef;
}

pub trait Component {
    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn properties(&self) -> Vec<PropertyDescriptor> {
        vec![PropertyDescriptor {
            name: String::from(""),
            description: None,
            data_type: PropertyType::F32,
        }]
    }
}
