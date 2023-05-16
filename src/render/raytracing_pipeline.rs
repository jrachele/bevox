use std::{sync::Arc, array};

use bevy::{math::IVec2, prelude::{Resource, Vec3}};
use bytemuck::{Zeroable, Pod};
use rand::Rng;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, CopyBufferInfo, PrimaryCommandBufferAbstract, ClearColorImageInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{DeviceOwned, Queue},
    format::Format,
    image::{ImageAccess, ImageUsage, StorageImage, ImageDimensions, view::ImageView},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout},
    sync::GpuFuture, DeviceSize,
};
use vulkano_util::renderer::DeviceImageView;

use crate::{voxel::{VoxelGrid, Voxel}, util::vary_color, WIDTH, HEIGHT, VOXEL_GRID_DIM, PlayerData};

/// Pipeline holding double buffered grid & color image.
/// Grids are used to calculate the state, and color image is used to show the output.
/// Because each step we determine state in parallel, we need to write the output to
/// another grid. Otherwise the state would not be correctly determined as one thread might read
/// data that was just written by another thread
#[derive(Resource)]
pub struct RayTracingComputePipeline {
    compute_queue: Arc<Queue>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    compute_pipeline: Arc<ComputePipeline>,
    voxel_buffer: Subbuffer<[u32]>,
    image: DeviceImageView
}

impl RayTracingComputePipeline {
    pub fn get_result_image(&self) -> DeviceImageView {
        self.image.clone()
    }
}

struct RayTracingPushConstants {

}

impl RayTracingComputePipeline {
    pub fn new(
        allocator: &Arc<StandardMemoryAllocator>,
        compute_queue: Arc<Queue>,
        voxel_buffer: Subbuffer<[u32]>
    ) -> RayTracingComputePipeline {
        let compute_pipeline = {
            let device = compute_queue.device();
            let shader = compute_render_cs::load(device.clone()).unwrap();
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

        let image = StorageImage::general_purpose_image_view(
            allocator,
            compute_queue.clone(),
            [WIDTH as u32, HEIGHT as u32],
            Format::R8G8B8A8_UNORM,
            ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST)
            .unwrap();

        RayTracingComputePipeline {
            compute_queue,
            command_buffer_allocator,
            descriptor_set_allocator,
            compute_pipeline,
            voxel_buffer,
            image
        }
    }

    pub fn compute(
        &mut self,
        before_future: Box<dyn GpuFuture>,
        player_data: PlayerData
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
        self.dispatch(&mut builder, &player_data);

        let command_buffer = builder.build().unwrap();
        let finished = before_future
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap();
        let after_pipeline = finished.then_signal_fence_and_flush().unwrap().boxed();

        after_pipeline
    }

    // Build the command for a dispatch.
    fn dispatch(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        player_data: &PlayerData
    ) {
        let pipeline_layout = self.compute_pipeline.layout();
        let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
        let set =
            PersistentDescriptorSet::new(&self.descriptor_set_allocator, desc_layout.clone(), [
                WriteDescriptorSet::buffer(0, self.voxel_buffer.clone()),
                WriteDescriptorSet::image_view(1, self.image.clone()),
            ])
            .unwrap();

        let push_constants = compute_render_cs::PushConstants {
            dim: VOXEL_GRID_DIM.into(),
            camera_matrix: player_data.camera_matrix.to_cols_array_2d(),
            inverse_projection_matrix: player_data.inverse_projection_matrix.to_cols_array_2d(),
            mouse_click: player_data.mouse_click,
            brush_size: player_data.brush_size
        };

        builder
            // .clear_color_image(ClearColorImageInfo {
            //     clear_value: vulkano::format::ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
            //     ..ClearColorImageInfo::image(self.image.image().clone())
            // }).unwrap()
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Compute, pipeline_layout.clone(), 0, set)
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .dispatch([WIDTH as u32 / 8, HEIGHT as u32 / 8, 1])
            .unwrap();
    }
}

mod compute_render_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/render.glsl",
    }
}
