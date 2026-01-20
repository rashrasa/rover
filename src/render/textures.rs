use std::collections::HashMap;

use image::{DynamicImage, ImageBuffer, imageops::FilterType};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource,
    Device, Extent3d, FilterMode, Origin3d, Queue, Sampler, SamplerDescriptor,
    TexelCopyBufferLayout, TexelCopyTextureInfoBase, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::MIPMAP_LEVELS;

#[derive(Clone)]
pub enum MipLevel {
    Square(u32),
}

pub enum Side2H {
    Left,
    Right,
}
pub enum Side2V {
    Up,
    Down,
}

/// Crop Side2 -> side which'll get trimmed out.
///
/// ShrinkToFit/Crop will fill in pixels with alpha = 0.0
pub enum ResizeStrategy {
    Crop(Side2H, Side2V),
    Stretch(FilterType),
    ShrinkToFit(FilterType),
}

type TextureEntry = (Texture, TextureView, Sampler, BindGroup);

#[derive(Debug)]
pub struct TextureStorage {
    textures: HashMap<String, TextureEntry>,
}

impl TextureStorage {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn get(&self, texture_id: &str) -> Option<&TextureEntry> {
        self.textures.get(texture_id)
    }

    // TODO: This currently ignores resize_strategy and just stretches.

    /// This will generate all mipmap levels for the texture.
    /// For [full_size_image], check crate::MIPMAP_LEVELS.
    pub fn new_texture(
        &mut self,
        device: &mut Device,
        queue: &mut Queue,
        texture_id: String,
        full_size_image: DynamicImage,
        resize_strategy: ResizeStrategy,
        bind_group_layout: &BindGroupLayout,
    ) {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("Texture: {}", texture_id)),
            size: Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1,
            },
            mip_level_count: MIPMAP_LEVELS.len() as u32,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let images: Vec<(MipLevel, ImageBuffer<image::Rgba<u8>, Vec<u8>>)> = MIPMAP_LEVELS
            .map(|level| match level {
                MipLevel::Square(size) => {
                    let start_width = full_size_image.width();
                    let start_height = full_size_image.height();
                    let delta_w = size as i64 - start_width as i64;
                    let delta_w = size as i64 - start_height as i64;

                    let image = full_size_image.resize_exact(size, size, FilterType::Gaussian);
                    (level, image.to_rgba8())
                }
            })
            .to_vec();

        for i in 0..images.len() {
            let level = i as u32;
            let (level_desc, image) = &images[i];
            queue.write_texture(
                TexelCopyTextureInfoBase {
                    texture: &texture,
                    mip_level: level,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                image,
                match level_desc {
                    MipLevel::Square(s) => TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(*s * 4),
                        rows_per_image: Some(*s),
                    },
                },
                match level_desc {
                    MipLevel::Square(s) => Extent3d {
                        width: *s,
                        height: *s,
                        depth_or_array_layers: 1,
                    },
                },
            );
        }
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("Texture Sampler: {}", texture_id)),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("Texture Bind Group: {}", texture_id)),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        self.textures
            .insert(texture_id, (texture, view, sampler, bind_group));
    }
}
