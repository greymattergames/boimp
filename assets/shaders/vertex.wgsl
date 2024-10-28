#import bevy_pbr::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput},
    view_transformations::{position_world_to_clip, direction_view_to_world, position_view_to_world},
    mesh_view_bindings::view,
}

struct ImposterData {
    center_and_scale: vec4<f32>,
    grid_size: u32,
    flags: u32,
}

@group(2) @binding(200)
var<uniform> imposter_data: ImposterData;
@group(2) @binding(201) 
var imposter_texture: texture_2d<f32>;
@group(2) @binding(202) 
var imposter_sampler: sampler;

struct ImposterVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) camera_direction: vec3<f32>,
    @location(2) base_world_position: vec3<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> ImposterVertexOut {
    var out: ImposterVertexOut;

    var model = mesh_functions::get_world_from_local(vertex.instance_index);

    let center = imposter_data.center_and_scale.xyz;
    let scale = imposter_data.center_and_scale.w;

    let imposter_world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(center, 1.0)).xyz;
    let camera_world_position = position_view_to_world(vec3<f32>(0.0));

    let back = normalize(camera_world_position - imposter_world_position);
    // not actually world normal, actually camera direction
    out.camera_direction = normalize(mesh_functions::mesh_position_local_to_world(model, vec4<f32>(back, 0.0))).xyz;

    let up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(up, back));
    let up2 = normalize(cross(back, right));
 
    let view_matrix = transpose(mat3x3(
        vec3(right),
        vec3(up2),
        vec3(back),
    ));

    out.base_world_position = imposter_world_position;
    out.world_position = imposter_world_position.xyz + (vertex.position * scale * 2.0) * view_matrix;
    out.position = position_world_to_clip(out.world_position);
    // out.uv = vertex.uv;
    // out.instance_index = vertex.instance_index;

    return out;
}
