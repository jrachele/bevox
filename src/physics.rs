use std::{sync::Arc, array};

use bevy::{math::IVec2, prelude::{Resource, Vec3}};
use bytemuck::{Zeroable, Pod};
use rand::Rng;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, CopyBufferInfo, PrimaryCommandBufferAbstract,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{DeviceOwned, Queue},
    format::Format,
    image::{ImageAccess, ImageUsage, StorageImage},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout},
    sync::GpuFuture, DeviceSize,
};
use vulkano_util::renderer::DeviceImageView;

use crate::{voxel::{VoxelGrid, Voxel}, util::vary_color, VOXEL_GRID_DIM};

/// Pipeline holding double buffered grid & color image.
/// Grids are used to calculate the state, and color image is used to show the output.
/// Because each step we determine state in parallel, we need to write the output to
/// another grid. Otherwise the state would not be correctly determined as one thread might read
/// data that was just written by another thread
#[derive(Resource)]
pub struct PhysicsComputePipeline {
    compute_queue: Arc<Queue>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    compute_pipeline: Arc<ComputePipeline>,
    voxel_buffer: Subbuffer<[u32]>,
    voxel_buffer_dbl: Subbuffer<[u32]>,
    // image: DeviceImageView,
}

impl PhysicsComputePipeline {
    pub fn get_voxel_grid(&self) -> &Subbuffer<[u32]> {
        &self.voxel_buffer
    }
}

fn create_voxel_grid(memory_allocator: &Arc<StandardMemoryAllocator>,
                     command_buffer_allocator: &StandardCommandBufferAllocator,
                     queue: Arc<Queue>) -> Subbuffer<[u32]> {

    // Create a sphere for testing purposes
    let n = VOXEL_GRID_DIM;
    let mut voxels = VoxelGrid::new(n, Vec3::default());
    let r = (n-1) as f32;
    let mut rng = rand::thread_rng();
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let pos = (Vec3::new(i as f32, j as f32, k as f32) * 2.0) - Vec3::splat((n) as f32);
                let in_sphere = pos.length_squared() <= r * r;
                if let Some(mut voxel) = voxels.get_mut(i, j, k) {
                    if in_sphere {
                        // For now just use sand and add nice variance
                        // TODO: Add model loading
                        // let is_sand = rng.gen_bool(0.5);

                        // if is_sand {
                            let variance = rng.gen_range(-0.02..0.02);
                            let sand_color = Vec3::new(0.5, 0.3,  0.1);
                            let varied_sand = vary_color(sand_color, variance);
                            voxel.set_color(varied_sand);
                        // }
                        // else {
                            // let water_color = Vec3::new(0.3, 0.7, 0.9);
                            // voxel.set_color(water_color);
                            // voxel.set_voxel_type(1);
                        // }
                    }
                }
            }
        }
    }

    // Create a host-accessible buffer initialized with the data.
    let temporary_accessible_buffer = Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            // Specify that this buffer will be used as a transfer source.
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            // Specify use for upload to the device.
            usage: MemoryUsage::Upload,
            ..Default::default()
        },
        voxels.voxels.clone().iter().map(|&x| {let u: u32 = x.into(); u})
    )
    .unwrap();

    let device_local_buffer = Buffer::new_slice::<u32>(
        memory_allocator,
        BufferCreateInfo {
            // Specify use as a storage buffer and transfer destination.
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST | BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            // Specify use by the device only.
            usage: MemoryUsage::DeviceOnly,
            ..Default::default()
        },
        voxels.total() as DeviceSize,
    )
    .unwrap();

    // Create a one-time command to copy between the buffers.
    let mut cbb = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    cbb.copy_buffer(CopyBufferInfo::buffers(
            temporary_accessible_buffer,
            device_local_buffer.clone(),
    )).unwrap();

    println!("Copy started");
    let command_buffer = cbb.build().unwrap();
    let future = command_buffer.execute(queue.clone()).unwrap().then_signal_fence_and_flush().unwrap();

    // Wait for the copy to happen
    future.wait(None).unwrap();
    println!("Copy finished");

    return device_local_buffer;
}

impl PhysicsComputePipeline {
    pub fn new(
        allocator: &Arc<StandardMemoryAllocator>,
        compute_queue: Arc<Queue>,
    ) -> PhysicsComputePipeline {
        let compute_pipeline = {
            let device = compute_queue.device();
            let shader = compute_physics_cs::load(device.clone()).unwrap();
            ComputePipeline::new(
                allocator.device().clone(),
                shader.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
                allocator.device().clone(),
                Default::default(),
        );

        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
                allocator.device().clone(),
        );

        let voxel_buffer = create_voxel_grid(allocator, &command_buffer_allocator, compute_queue.clone());
        let voxel_buffer_dbl = create_voxel_grid(allocator, &command_buffer_allocator, compute_queue.clone());

        PhysicsComputePipeline {
            compute_queue,
            command_buffer_allocator,
            descriptor_set_allocator,
            compute_pipeline,
            voxel_buffer,
            voxel_buffer_dbl,
        }
    }

    pub fn compute(
        &mut self,
        before_future: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Dispatch will mutate the builder adding commands which won't be sent before we build the command buffer
        // after dispatches. This will minimize the commands we send to the GPU. For example, we could be doing
        // tens of dispatches here depending on our needs. Maybe we wanted to simulate 10 steps at a time...

        // First compute the next state
        self.dispatch(&mut builder);

        let command_buffer = builder.build().unwrap();
        let finished = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap();
        let after_pipeline = finished.then_signal_fence_and_flush().unwrap();

        // std::mem::swap(&mut self.voxel_buffer, &mut self.voxel_buffer_dbl);
        // after_pipeline.wait(None).unwrap();

        self.swap_and_clean(after_pipeline.boxed())
        // after_pipeline.boxed()
    }

    fn swap_and_clean(&self, future: Box<dyn GpuFuture>) -> Box<dyn GpuFuture> {

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        builder
            .copy_buffer(CopyBufferInfo::buffers(self.voxel_buffer_dbl.clone(), self.voxel_buffer.clone()))
            .unwrap();
        let command_buffer = builder.build().unwrap();
        let copy_future = future.then_execute(self.compute_queue.clone(), command_buffer).unwrap().then_signal_fence_and_flush().unwrap();

        copy_future.boxed()
        // let mut builder = AutoCommandBufferBuilder::primary(
        //     &self.command_buffer_allocator,
        //     self.compute_queue.queue_family_index(),
        //     CommandBufferUsage::OneTimeSubmit
        // ).unwrap();

        // builder
        //     .fill_buffer(self.voxel_buffer_dbl.clone(), 0)
        //     .unwrap();
        // let command_buffer = builder.build().unwrap();
        // let fill_future = copy_future.then_execute(self.compute_queue.clone(), command_buffer).unwrap().then_signal_fence_and_flush().unwrap();

        // fill_future.boxed()
    }

    // Build the command for a dispatch.
    fn dispatch(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let pipeline_layout = self.compute_pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set =
            PersistentDescriptorSet::new(&self.descriptor_set_allocator, desc_layout.clone(), [
                WriteDescriptorSet::buffer(0, self.voxel_buffer.clone()),
                WriteDescriptorSet::buffer(1, self.voxel_buffer_dbl.clone()),
            ])
            .unwrap();

        let push_constants = compute_physics_cs::PushConstants {
            dim: VOXEL_GRID_DIM,
        };
        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([VOXEL_GRID_DIM / 8, VOXEL_GRID_DIM / 8, VOXEL_GRID_DIM / 8])
            .unwrap();
    }
}

mod compute_physics_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/physics.glsl",
    }
}
