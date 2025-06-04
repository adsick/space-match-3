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

const PARTICLE_RADIUS: f32 = 3.0;

fn snoise3(p: vec3<f32>) -> vec3<f32> {
    return vec3f(snoise(p), snoise(p + vec3f(100.0)), 0.0);
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

    let local_point = point - center;
    let dist_to_engine = distance(point, center);

    var density = 0.0;
    var prev_dist = 1.0;
    for (var i = 0u; i < nof_particles; i++) {
        var particle_pos = particles[i].xyz;
        particle_pos += snoise3(particle_pos) * dist_to_engine * 0.1;

        let particle_intensity = (1.0 - f32(i) / f32(nof_particles));
        let particle_radius = PARTICLE_RADIUS * particle_intensity;

        var dist = distance(point, particle_pos);

        dist = smoothstep(0.0, particle_radius, dist);
        let t = dist;
        dist = smooth_min(dist, prev_dist * (1.1 + 2 * particle_intensity), 0.9);
        prev_dist = t;

        var curr_density = 1.0 - dist;

        curr_density = smooth_min(curr_density, dist, 0.9);

        density += curr_density;
    }

    density = smoothstep(0.0, 1.0, density * power);


    var mask = dot(-dir.xy, normalize(local_point.xy));
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

    out.color = mix(vec4f(color, 0.0), vec4f(1.0), density);

    // if (density < 0.7 && density > 0.4) {
    //   out.color = mix(vec4f(color, 0.7), vec4f(1.0), 0.9);
    // }

    if (density > 0.7) {
      out.color = mix(vec4f(color, 0.7), vec4f(1.0), 0.9);
    } else if (density > 0.5) {
      out.color = mix(vec4f(color, 0.7), vec4f(1.0), 0.7);
    } else  if (density > 0.3) {
      out.color = mix(vec4f(color, 0.5), vec4f(1.0), 0.5);
    } else if (density > 0.1) {
      out.color = mix(vec4f(color, 0.4), vec4f(1.0), 0.3);
    } else {
      out.color = vec4f(0.0);
    }

    return out;




    // pbr_input.material.base_color.b = pbr_input.material.base_color.r;
    // pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    // var out: FragmentOutput;
    // out.color = apply_pbr_lighting(pbr_input);
    // out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    // out.color = out.color * 2.0;
    // return out;
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
