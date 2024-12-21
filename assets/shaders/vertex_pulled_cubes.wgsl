// `custom_phase_item.wgsl`
//
// This shader goes with the `custom_phase_item` example. It demonstrates how to
// enqueue custom rendering logic in a `RenderPhase`.

#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0) var<storage, read> triangles: array<mat4x4f>;

// Information passed from the vertex shader to the fragment shader.
struct VertexOutput {
    // The clip-space position of the vertex.
    @builtin(position) clip_position: vec4<f32>,  
    // The color of the vertex.
    @location(0) color: vec3<f32>,
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

const VERTEX_COUNT: u32 = 36;

fn get_base_vertex(index: u32) -> vec4f {
  return vec4f(VERTICES[index % VERTEX_COUNT], 1.0);
}

// The vertex shader entry point.
@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    // Use an orthographic projection.
    var vertex_output: VertexOutput;
    var vertex_pos: vec4f = get_base_vertex(index);
    vertex_pos *= triangles[index / VERTEX_COUNT];
    let transform = triangles[index / VERTEX_COUNT];
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
    return vertex_output;
}

// The fragment shader entry point.
@fragment
fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(vertex_output.color, 1.0);
}
