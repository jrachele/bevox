use bevy::{prelude::Vec3, render::render_resource::ShaderType};
use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug, Default)]
pub enum VoxelType {
    #[default]
    Grass,
    Dirt
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Voxel(u32);

impl Into<u32> for Voxel {
    fn into(self) -> u32 {
        self.0
    }
}

impl Voxel {
    pub fn get_color(&self) -> Vec3 {
        let voxel_data = self.0;
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
        self.0 |= r << (32 - 5);
        self.0 |= g << (32 - 11);
        self.0 |= b << (32 - 16);
    }

    pub fn get_voxel_type(&self) -> u32 {
        return self.0 & 255;
    }

    pub fn set_voxel_type(&mut self, voxel_type: u32) {
        self.0 >>= 8;
        self.0 <<= 8;
        self.0 |= voxel_type & 255;
    }

}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct VoxelGrid {
    dim: u32,
    pos: Vec3,
    selected: Vec3,
    normal: Vec3,
    pub voxels: Vec<Voxel>,
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

    pub fn total(&self) -> usize {
        (self.dim * self.dim * self.dim) as usize
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

    pub fn as_slice(&self) -> &[Voxel] {
        self.voxels.as_slice()
    }
}
