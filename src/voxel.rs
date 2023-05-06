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
    pub value: u32
}

impl Voxel {
    fn get_color(&self) -> (f32, f32, f32) {
        //
        let voxel_type_num = self.value >> 16;
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

#[derive(Clone, Debug, Default, ShaderType)]
pub struct VoxelGrid {
    dim: u32,
    pub pos: Vec3,
    #[size(runtime)]
    voxels: Vec<Voxel>,
}

impl VoxelGrid {
	pub fn new(dim: u32, pos: Vec3) -> Self {
        Self {
            dim,
            pos,
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
