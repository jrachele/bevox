use bevy::prelude::Vec3;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug, Default)]
pub enum VoxelType {
    #[default]
    Grass,
    Dirt
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Voxel(pub u32);

impl Voxel {
    fn get_color(&self) -> (f32, f32, f32) {
        //
        let voxel_type_num = self.0 >> 16;
        let voxel_type = match voxel_type_num {
            0 => VoxelType::Grass,
            _ => VoxelType::Dirt,
        };
        Self::color_from_voxel_type(voxel_type)
    }

    fn color_from_voxel_type(voxel_type: VoxelType) -> (f32, f32, f32) {
        match voxel_type {
            VoxelType::Grass => (0.0, 0.9, 0.1),
            VoxelType::Dirt => (0.7, 0.3, 0.1),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct VoxelGrid {
    pub voxels: Vec<Voxel>,
    dim: usize,
}

impl VoxelGrid {
	pub fn new(dim: usize) -> Self {
        Self {
            voxels: vec![Voxel(0); dim*dim*dim],
            dim,
        }
	}

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&Voxel> {
        let index = (x * self.dim * self.dim) + (y * self.dim) + z;
        if index >= self.voxels.len() {
            return None;
        }
        Some(&self.voxels[index])
    }

    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Voxel> {
        let index = (x * self.dim * self.dim) + (y * self.dim) + z;
        if index >= self.voxels.len() {
            return None;
        }
        Some(&mut self.voxels[index])
    }
}
