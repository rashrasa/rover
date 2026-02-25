struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct LightUniform {
    pos: vec4<f32>,
    colour: vec4<f32>,
    luminence: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var s: sampler;

@group(2) @binding(0)
var<uniform> light: LightUniform;

@group(3) @binding(0)
var depth_texture: texture_depth_2d;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(5) x: vec4<f32>,
    @location(6) y: vec4<f32>,
    @location(7) z: vec4<f32>,
    @location(8) w: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let transform = mat4x4<f32>(
        instance.x,
        instance.y,
        instance.z,
        instance.w
    );
    var out: VertexOutput;
    out.world_position = transform * vec4<f32>(model.position, 1.0);
    out.normal = mat3x3<f32>(transform[0].xyz, transform[1].xyz, transform[2].xyz) * model.normal;
    out.clip_position = camera.view_proj * transform * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_pos = light.pos.xyz;
    let light_vec = in.world_position.xyz - light_pos;
    let light_dist = length(light_vec);
    let light_unit_vec = normalize(light_vec);
    let brightness = light.luminence * 1.0 / max(light_dist * light_dist, 1.0);
    let lighting = light.colour.xyz * max(dot(in.normal, -light_unit_vec), 0.0);

    return vec4<f32>(lighting * brightness, 1.0) * textureSample(texture, s, in.tex_coords);
}

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let depth = textureLoad(depth_texture, vec2i(floor(in.world_position.xy)), 0);

//     return vec4<f32>(1.0 - depth,1.0 - depth,1.0 - depth,1.0);
// }