// `custom_phase_item.wgsl`
//
// This shader goes with the `custom_phase_item` example. It demonstrates how to
// enqueue custom rendering logic in a `RenderPhase`.


#import bevy_render::view::View;

#import bevy_pbr::{
    forward_io::FragmentOutput,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing, calculate_view},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    mesh_types::{MESH_FLAGS_TRANSMITTED_SHADOW_RECEIVER_BIT, MESH_FLAGS_SHADOW_RECEIVER_BIT},
};

#import bevy_pbr::{
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_types::pbr_input_new,
    shadows,
};

@group(0) @binding(0)
var<uniform> view: View;

struct CubeArray {
  arr: array<Cube>,
}

struct Cube {
  transform: mat4x4f,
}

@group(1) @binding(0)
var<storage, read> chunks: binding_array<CubeArray>;

// Information passed from the vertex shader to the fragment shader.
struct VertexOutput {
    // The clip-space position of the vertex.
    @builtin(position) clip_position: vec4<f32>,
    // The color of the vertex.
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec4f,
};

const VERTICES = array<vec3f, 36>(

// +X

  vec3f( 0.5,  0.5, -0.5),
  vec3f( 0.5,  0.5,  0.5),
  vec3f( 0.5, -0.5, -0.5),
  vec3f( 0.5,  0.5,  0.5),
  vec3f( 0.5, -0.5,  0.5),
  vec3f( 0.5, -0.5, -0.5),

// -X

  vec3f(-0.5,  0.5, -0.5),
  vec3f(-0.5, -0.5, -0.5),
  vec3f(-0.5,  0.5,  0.5),
  vec3f(-0.5,  0.5,  0.5),
  vec3f(-0.5, -0.5, -0.5),
  vec3f(-0.5, -0.5,  0.5),

// +Y

  vec3f(-0.5, 0.5,  0.5),
  vec3f( 0.5, 0.5,  0.5),
  vec3f(-0.5, 0.5, -0.5),
  vec3f( 0.5, 0.5,  0.5),
  vec3f( 0.5, 0.5, -0.5),
  vec3f(-0.5, 0.5, -0.5),

// -Y

  vec3f(-0.5, -0.5,  0.5),
  vec3f(-0.5, -0.5, -0.5),
  vec3f( 0.5, -0.5,  0.5),
  vec3f( 0.5, -0.5,  0.5),
  vec3f(-0.5, -0.5, -0.5),
  vec3f( 0.5, -0.5, -0.5),

  // +Z

  vec3f(-0.5,  0.5, 0.5),
  vec3f( 0.5,  0.5, 0.5),
  vec3f(-0.5, -0.5, 0.5),
  vec3f( 0.5,  0.5, 0.5),
  vec3f( 0.5, -0.5, 0.5),
  vec3f(-0.5, -0.5, 0.5),

// -Z

  vec3f(-0.5,  0.5, -0.5),
  vec3f(-0.5, -0.5, -0.5),
  vec3f( 0.5,  0.5, -0.5),
  vec3f( 0.5,  0.5, -0.5),
  vec3f(-0.5, -0.5, -0.5),
  vec3f( 0.5, -0.5, -0.5),
);

const NORMALS = array<vec3f, 6> (
  vec3f( 1.0, 0.0, 0.0),
  vec3f(-1.0, 0.0, 0.0),
  vec3f( 0.0, 1.0, 0.0),
  vec3f( 0.0,-1.0, 0.0),
  vec3f( 0.0, 0.0, 1.0),
  vec3f( 0.0, 0.0,-1.0),
);

const TEST_AXIS = array<vec4f, 3> (
    vec4f(1.0, 0.0, 0.0, 1.0),
    vec4f(0.0, 1.0, 0.0, 1.0),
    vec4f(0.0, 0.0, 1.0, 1.0),
);

const VERTEX_COUNT: u32 = 18;

fn get_base_vertex(index: u32) -> vec4f {
  return vec4f(VERTICES[index % VERTEX_COUNT], 1.0);
}

fn get_base_normal(index: u32) -> vec3f {
  return NORMALS[(index / 6) % 6];
}

// The vertex shader entry point.
@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    // Use an orthographic projection.
    var vertex_output: VertexOutput;

    let chunk_num = index % 2;

    var instance = index / VERTEX_COUNT;
    instance += 0;
    let axis = (index - instance) / 6;
    var cube_pos = (vec4(0, 0, 0, 1) * chunks[chunk_num].arr[instance].transform)[axis];
    var axis_test = (TEST_AXIS[axis] * chunks[chunk_num].arr[instance].transform)[axis];
    let sign = sign(cube_pos);
    axis_test *= sign;
    cube_pos *= sign;
    var vertex = axis * 12 + (index % 6);
    if (f32(axis) > cube_pos) {
        vertex = vertex + 6;
    }

    var vertex_pos: vec4f = get_base_vertex(vertex);
    vertex_pos = vertex_pos *
                 chunks[chunk_num].arr[index / VERTEX_COUNT].transform;
    vertex_output.world_position = vertex_pos;
    vertex_pos = view.clip_from_world * vertex_pos;
    vertex_output.clip_position = vertex_pos;
    let i = index % 3;
    if (i == 0) {
      vertex_output.color = vec3f(1.0, 0.0, 0.0);
    } else if (i == 1) {
      vertex_output.color = vec3f(0.0, 1.0, 0.0);
    } if (i == 2) {
      vertex_output.color = vec3f(0.0, 0.0, 1.0);
    }
    vertex_output.normal = get_base_normal(index);

    vertex_output.color = vec3(0.5, 0.5, 1.0);

    return vertex_output;
}

// The fragment shader entry point.
@fragment
fn fragment(vertex: VertexOutput) -> @location(0) vec4f {
    var pbr_input = pbr_input_new();

    pbr_input.frag_coord = vertex.clip_position;
    pbr_input.world_position = vertex.world_position;
    pbr_input.world_normal = vertex.normal;
    pbr_input.N = vertex.normal.xyz;
    pbr_input.V = calculate_view(vertex.world_position, false);

    // pbr_input.material.base_color = vec4f(0.5, 0.5, 1.0, 1.0);
    pbr_input.material.base_color = vec4(vertex.color, 1.0);
    pbr_input.material.perceptual_roughness = 0.05;

    pbr_input.material.base_color = alpha_discard(
      pbr_input.material,
      pbr_input.material.base_color
    );

    pbr_input.flags |= MESH_FLAGS_SHADOW_RECEIVER_BIT;

    var out: FragmentOutput;
    // if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    // } else {
        // out.color = pbr_input.material.base_color;
    // }

    return main_pass_post_lighting_processing(pbr_input, out.color);
}

// @vertex
// fn shadow_vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
//     var vertex_pos = get_base_vertex(index);
//     vertex_pos *= cuboids[index / VERTEX_COUNT];
//     vertex_pos = view.clip_from_world * vertex_pos;
//     return vertex_pos;
// }
// 
// @fragment
// fn shadow_fragment(@builtin(position) clip_pos: vec4f) {
//     return;
// }
