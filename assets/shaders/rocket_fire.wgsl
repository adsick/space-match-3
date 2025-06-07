#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_view_bindings,
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

@group(2) @binding(100)
var<uniform> color: vec4f;
@group(2) @binding(101)
var<uniform> center: vec4f;
@group(2) @binding(102)
var<uniform> nof_particles: vec4u;
@group(2) @binding(103)
var<uniform> particles: array<vec4f, 32>;
@group(2) @binding(104)
var<uniform> dir: vec4f;
@group(2) @binding(105)
var<uniform> power: vec4f;

const PARTICLE_RADIUS: f32 = 5.0;

fn snoise2(p: vec2<f32>) -> vec2<f32> {
    return vec2f(snoise(vec3f(p, 0.0)), snoise(vec3f(p, 0.0) + vec3f(100.0)));
}


fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

// fn smooth_max(a: f32, b: f32, k: f32) -> f32 {
//     let h = clamp(0.5 + 0.5 * (a - b) / k, 0.0, 1.0);
//     return a * h + b * (1.0 - h) + k * h * (1.0 - h);
// }



fn density_at_point(point: vec3f) -> f32 {
    let t = mesh_view_bindings::globals.time;

    let color = color.xyz;
    let center = center.xyz;
    let nof_particles = nof_particles.x;
    let power = power.x;

    let noise_point = smoothstep(0.0, 30., distance(point, center)) * 1.0 * snoise2(point.xy) + point.xy;

    // SDF to polyline
    var min_dist = 1e10;
    let count = i32(nof_particles);
    var closest_particle_i = 0;

    for (var i = 0; i < count - 1; i = i + 1) {
        let a = particles[i].xy;
        let b = particles[i + 1].xy;

        let pa = noise_point - a;
        let ba = b - a;
        let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        let d = length(pa - ba * h);

        if (min_dist > d) {
            min_dist = d;
            closest_particle_i = i;
        }
    }

    let dist = smoothstep(0.0, PARTICLE_RADIUS, min_dist);
    var density = 1.0 - dist;


    let last_particle_dist = distance(particles[nof_particles].xy, center.xy);
    let particle_intensity = smoothstep(1.0, 0.0, f32(closest_particle_i) / f32(nof_particles));
    density *= particle_intensity;


    var mask = dot(-dir.xy, normalize(point - center).xy);
    mask = smoothstep(0.4, 0.9, mask);
    density =  density * mask;

    return density;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var color = color.xyz;
    let center = center.xyz;
    let power = power.x;


    let pbr_input = pbr_input_from_standard_material(in, is_front);
    // let light_direction = mesh_view_bindings::lights.directional_lights[0].direction_to_light;

    var position = pbr_input.world_position.xyz;

    var out: FragmentOutput;

    var density = 0.0;

    density += density_at_point(position);
    density *= power;

    // out.color = mix(vec4f(0.0), vec4f(color, 1.0), density);
    if (density > 0.4) {
        out.color = vec4f(5.0);
    } else if (density > 0.2) {
        out.color = vec4f(color, 1.0);
    } else if (density > 0.1) {
        out.color = mix(vec4f(1.0), vec4f(color, 1.0), density * 5.);
    } else {
        out.color = mix(vec4f(0.0), vec4f(color, 1.0), density);
    }


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


fn snoiseFractal(m: vec3<f32>) -> f32 {
    return 0.5333333 * snoise(m)
         + 0.2666667 * snoise(2.0 * m)
         + 0.1333333 * snoise(4.0 * m)
         + 0.0666667 * snoise(8.0 * m);
}
