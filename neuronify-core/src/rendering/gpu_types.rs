use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};
use visula_derive::Instance;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Instance, Pod, Zeroable)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub _padding: f32,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct ConnectionData {
    pub start_color: Vec3,
    pub end_color: Vec3,
    pub position_a: Vec3,
    pub position_b: Vec3,
    pub strength: f32,
    pub directional: f32,
    pub _padding: [f32; 2],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct LineData {
    pub start: Vec3,
    pub end: Vec3,
    pub _padding: [f32; 2],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Instance, Pod, Zeroable)]
pub struct MeshInstanceData {
    pub position: Vec3,
    pub _padding: f32,
    pub rotation: Quat,
}
