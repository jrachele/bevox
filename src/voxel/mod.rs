use bevy::{prelude::Vec3, render::render_resource::ShaderType};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub enum VoxelType {
    #[default]
    Grass,
    Dirt
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct Voxel {
    value: u32
}

impl Voxel {
    pub fn get_color(&self) -> Vec3 {
        let voxel_data = self.value;
        let mask_5 = 31;
        let mask_6 = 63;
        let r_offset = 32 - 5;
        let g_offset = 32 - 11;
        let b_offset = 32 - 16;
        let r: u32 = (voxel_data & (mask_5 << r_offset)) >> r_offset;
        let g: u32 = (voxel_data & (mask_6 << g_offset)) >> g_offset;
        let b: u32 = (voxel_data & (mask_5 << b_offset)) >> b_offset;
        Vec3::new(r as f32 / mask_5 as f32, g as f32 / mask_6 as f32, b as f32 / mask_5 as f32)
    }

    pub fn set_color(&mut self, color: Vec3) {
        let r: u32 = (color.x.abs() * 31.0) as u32;
        let g: u32 = (color.y.abs() * 63.0) as u32;
        let b: u32 = (color.z.abs() * 31.0) as u32;
        self.value |= r << (32 - 5);
        self.value |= g << (32 - 11);
        self.value |= b << (32 - 16);
    }

    pub fn get_voxel_type(&self) -> u32 {
        return self.value & 255;
    }

    pub fn set_voxel_type(&mut self, voxel_type: u32) {
        self.value >>= 8;
        self.value <<= 8;
        self.value |= voxel_type & 255;
    }

}

#[derive(Clone, Debug, Default, ShaderType)]
pub struct VoxelGrid {
    dim: u32,
    pub pos: Vec3,
    selected: Vec3,
    normal: Vec3,
    #[size(runtime)]
    voxels: Vec<Voxel>,
}

impl VoxelGrid {
	pub fn new(dim: u32, pos: Vec3) -> Self {
        Self {
            dim,
            pos,
            normal: Vec3::new(0.0, 0.0, 0.0),
            selected: Vec3::new(-1.0, -1.0, -1.0),
            voxels: vec![Voxel::default(); (dim*dim*dim) as usize],
        }
	}

    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<&Voxel> {
        let index = (x * self.dim * self.dim) + (y * self.dim) + z;
        if index >= self.voxels.len() as u32 {
            return None;
        }
        Some(&self.voxels[index as usize])
    }

    pub fn get_mut(&mut self, x: u32, y: u32, z: u32) -> Option<&mut Voxel> {
        let index = (x * self.dim * self.dim) + (y * self.dim) + z;
        if index >= self.voxels.len()  as u32{
            return None;
        }
        Some(&mut self.voxels[index as usize])
    }
}
