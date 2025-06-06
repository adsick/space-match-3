#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings,
    view_transformations::position_world_to_clip,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // @location(3) terrain_seed: vec4f,

    // @location(3) tangent: vec4<f32>,
}



@group(2) @binding(100)
var<uniform> terrain_seed: vec4f;
@group(2) @binding(101)
var<uniform> radius: vec4f;


const HEIGHT_SCALE: f32 = 0.2;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let radius = radius.x;

    let world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4f(in.position, 1.0));

    let terrain_height = snoise(in.position  / 50. + terrain_seed.xyz) * 2. - 1.;
    let displaced_pos = in.position + in.normal * terrain_height * radius * HEIGHT_SCALE;
    let local_position = vec4(displaced_pos, 1.0);

    var out: VertexOutput;

    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, local_position);

    out.position = position_world_to_clip(out.world_position.xyz);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(in.normal, in.instance_index);
    out.uv = in.uv;
    out.instance_index = in.instance_index;

    return out;
}




@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    out.color = smoothstep(vec4f(0.03), vec4f(0.2), out.color);
    out.color.w = 1.0;

    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}


// Translated from Shadertoy: <www.shadertoy.com/view/XsX3zB>
// by Nikita Miropolskiy

/* discontinuous pseudorandom uniformly distributed in [-0.5, +0.5]^3 */
fn random3(c: vec3<f32>) -> vec3<f32> {
    let j = 4096.0 * sin(dot(c, vec3<f32>(17.0, 59.4, 15.0)));
    var r: vec3<f32>;
    r.z = fract(512.0 * j);
    let j1 = j * 0.125;
    r.x = fract(512.0 * j1);
    let j2 = j1 * 0.125;
    r.y = fract(512.0 * j2);
    return r - 0.5;
}

const F3: f32 = 0.3333333;
const G3: f32 = 0.1666667;

fn snoise(p: vec3<f32>) -> f32 {
    let s = floor(p + dot(p, vec3<f32>(F3)));
    let x = p - s + dot(s, vec3<f32>(G3));

    let e = step(vec3<f32>(0.0), x - x.yzx);
    let i1 = e * (1.0 - e.zxy);
    let i2 = 1.0 - e.zxy * (1.0 - e);

    let x1 = x - i1 + G3;
    let x2 = x - i2 + 2.0 * G3;
    let x3 = x - 1.0 + 3.0 * G3;

    var w: vec4<f32>;
    var d: vec4<f32>;

    w.x = dot(x, x);
    w.y = dot(x1, x1);
    w.z = dot(x2, x2);
    w.w = dot(x3, x3);

    w = max(0.6 - w, vec4<f32>(0.0));

    d.x = dot(random3(s), x);
    d.y = dot(random3(s + i1), x1);
    d.z = dot(random3(s + i2), x2);
    d.w = dot(random3(s + vec3<f32>(1.0)), x3);

    w = w * w;
    w = w * w;
    d = d * w;

    return dot(d, vec4<f32>(52.0));
}
