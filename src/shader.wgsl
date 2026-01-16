struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct InstanceInput {
    @location(5) x: vec4<f32>,
    @location(6) y: vec4<f32>,
    @location(7) z: vec4<f32>,
    @location(8) w: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
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
    out.color = model.color;
    out.clip_position = camera.view_proj * transform * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}