// Vertex shader

struct Camera {
    view_pos: vec4<f32>,
    view_mat: mat4x4<f32>,
    proj_mat: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    let world_position = vec4<f32>(model.position * 10.0, 1.0);

    let view3x3=mat3x3<f32>(camera.view_mat[0].xyz,camera.view_mat[1].xyz,camera.view_mat[2].xyz);
    let view_from_origin=mat4x4<f32>(
        vec4<f32>(view3x3[0].xyz,0.0),
        vec4<f32>(view3x3[1].xyz,0.0),
        vec4<f32>(view3x3[2].xyz,0.0),
        vec4<f32>(0.0,0.0,0.0,1.0),
    );

    var out: VertexOutput;
    out.clip_position = camera.proj_mat * view_from_origin * world_position;
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, vec2<f32>(in.tex_coords.x,1.0-in.tex_coords.y));

    return object_color;
}
