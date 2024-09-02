#import bevy_pbr::mesh_vertex_output MeshVertexOutput
#import bevy_render::view View

struct ParticleMaterial {
    color: vec4<f32>,
    time: f32,
}

@group(1) @binding(0) var<uniform> material: ParticleMaterial;
@group(0) @binding(0) var<uniform> view: View;

@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
) -> MeshVertexOutput {
    var out: MeshVertexOutput;
    out.world_position = vec4<f32>(position, 1.0);
    out.position = view.view_proj * out.world_position;
    out.uv = uv;
    return out;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    // Use UV coordinates for radial fade
    let center = vec2(0.5, 0.5);
    let dist = distance(in.uv, center);

    // Noise displacement
    let noise = sin(in.uv.x * 10.0 + material.time) * cos(in.uv.y * 10.0 + material.time) * 0.5 + 0.5;

    // Combine color, radial fade, and noise
    var final_color = material.color;
    final_color.a *= smoothstep(1.0, 0.0, dist * 2.0);
    final_color.rgb += vec3(noise) * 0.2;

    // Time-based fade out
    final_color.a *= 1.0 - (material.time / 1.5); // Assuming max lifetime is 1.5 seconds

    return final_color;
}
