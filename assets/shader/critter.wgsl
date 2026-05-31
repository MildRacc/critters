#import bevy_sprite::mesh2d_vertex_output::VertexOutput



struct GpuCritter
{
    position: vec2<f32>,
    velocity: vec2<f32>,
    radius: f32,
    color: u32,
    _padding: vec2<f32>
}



@group(2) @binding(0) var<storage, read> critters: array<GpuCritter>;
@group(2) @binding(1) var<uniform> critter_count: u32;
@group(2) @binding(2) var<uniform> resolution: vec2<f32>;
@group(2) @binding(3) var<uniform> cam_offset: vec2<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32>
{

    let pixel = in.uv * resolution;
    var out_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
 
    var met: f32 = 0.0;
    var weighted_color: vec4<f32> = vec4<f32>(0.0);
    var total_contribution: f32 = 0.0;

    for(var i = 0u; i < critter_count; i++)
    {

        let c = critters[i];
        let dist = distance(pixel, c.position - cam_offset);

        let contribution = 1.0 / (dist*dist);
        met += contribution;

        weighted_color += COLORS[c.color] * contribution;
        total_contribution += contribution;
    }

    let color = weighted_color / total_contribution;

    let inner = smoothstep(0.08, 0.12, met);
    let outer = smoothstep(0.12, 0.2, met);

    let membrane_color = vec4<f32>(color.xyz / 4.0, 1.0);
    let blob_color = vec4<f32>(color.xyz * inner, inner);

    return mix(blob_color, membrane_color, outer-inner);

}



struct CritterUniforms
{
    dt: f32,
    damping: f32,
    critter_count: u32
}

@group(0) @binding(0) var<storage, read_write> critters_in: array<GpuCritter>;
@group(0) @binding(1) var<storage, read_write> critters_out: array<GpuCritter>;
@group(0) @binding(2) var<uniform> uniforms: CritterUniforms;

@compute @workgroup_size(64)
fn physics_compute(@builtin(global_invocation_id) global_id: vec3<u32>)
{

    let i = global_id.x;
    if(i >= uniforms.critter_count) {return;}

    var critter = critters_in[i];
    var velocity = critter.velocity;

    for(var c = 0u; c < uniforms.critter_count; c++)
    {

        if c == i {continue;}

        let other = critters_in[c];
        let diff = other.position - critter.position;
        let dist = length(diff);
        if dist < 0.001 {continue;}

        let dir = diff / dist;

        let force = calculate_force(critter.color, other.color, critter.radius, other.radius, dist);

        velocity += dir * force * uniforms.dt;

    }

    velocity *= (1.0 - uniforms.damping);

    critters_out[i].position = critter.position + velocity * uniforms.dt;
    critters_out[i].velocity = velocity;
    critters_out[i].radius = critter.radius;
    critters_out[i].color = critter.color;

}


fn calculate_force(color: u32, other_color: u32, rad: f32, other_rad: f32, dist: f32) -> f32
{

    let s = STATS[color][other_color];
    let min_inner     = s[0];
    let orbit_radius  = s[1];
    let perception    = s[2];
    let repel_force   = s[3];
    let attract_force = s[4];
    

    
    if dist > perception{ return 0.0; }

    // zone 1: too close — hard repel
    if dist < min_inner {
        let t = 1.0 - (dist / min_inner);
        return -repel_force * t; 
    }

    // zone 2: between repel and orbit — attract toward orbit
    if dist < orbit_radius {
        let t = (dist - min_inner) / (orbit_radius - min_inner);
        return attract_force * (1.0 - t); // pull inward
    }

    // zone 3: between orbit and perception — attract back toward orbit
    let t = (dist - orbit_radius) / (perception - orbit_radius);
    return attract_force * (1.0 - t); // weaker pull as they get farther


}

    


// [self_color][other_color][stat_index]
// stats: 0=min_inner, 1=orbit_radius, 2=perception, 3=repel_force, 4=attract_force
const STATS: array<array<array<f32, 5>, 8>, 8> = array(
    // A
    array(
        array(34.0, 36.0,  80.0, 16.0, 15.0), // A vs A
        array(34.0, 38.0, 130.0, 13.0, 12.0), // A vs B
        array(34.0, 36.0,  55.0, 15.0,-15.0), // A vs C
        array(34.0, 36.0,  55.0, 15.0,-15.0), // A vs D
        array(34.0, 36.0,  55.0, 15.0,-15.0), // A vs E
        array(34.0, 36.0,  55.0, 15.0,-15.0), // A vs F
        array(34.0, 36.0,  55.0, 15.0,-15.0), // A vs G
        array(34.0, 36.0,  55.0, 15.0,-15.0), // A vs H
    ),
    // B
    array(
        array(40.0, 55.0, 130.0, 13.0, 12.0), // B vs A
        array(34.0, 40.0, 100.0, 12.0, 10.0), // B vs B
        array(34.0, 42.0,  90.0, 11.0,  8.0), // B vs C
        array(34.0, 38.0,  65.0, 12.0,-10.0), // B vs D
        array(34.0, 38.0,  65.0, 12.0,-10.0), // B vs E
        array(34.0, 38.0,  65.0, 12.0,-10.0), // B vs F
        array(34.0, 38.0,  65.0, 12.0,-10.0), // B vs G
        array(34.0, 38.0,  65.0, 12.0,-10.0), // B vs H
    ),
    // C
    array(
        array(55.0,  90.0, 180.0, 10.0,  6.0), // C vs A
        array(45.0,  65.0, 150.0, 10.0,  9.0), // C vs B
        array(34.0,  44.0, 120.0, 10.0,  7.0), // C vs C
        array(34.0,  42.0,  90.0, 10.0,  5.0), // C vs D
        array(34.0,  42.0,  80.0,  9.0, -6.0), // C vs E
        array(34.0,  42.0,  80.0,  9.0, -6.0), // C vs F
        array(34.0,  42.0,  80.0,  9.0, -6.0), // C vs G
        array(34.0,  42.0,  80.0,  9.0, -8.0), // C vs H
    ),
    // D
    array(
        array(80.0, 130.0, 260.0,  9.0,  6.0), // D vs A
        array(70.0, 110.0, 220.0,  9.0,  7.0), // D vs B
        array(55.0,  85.0, 180.0,  9.0,  6.0), // D vs C
        array(45.0,  65.0, 180.0, 10.0, -5.0), // D vs D
        array(40.0,  60.0, 140.0,  9.0, -8.0), // D vs E
        array(40.0,  60.0, 140.0,  9.0, -8.0), // D vs F
        array(40.0,  60.0, 140.0,  9.0, -8.0), // D vs G
        array(40.0,  60.0, 140.0,  9.0, -8.0), // D vs H
    ),
    // E
    array(
        array(110.0, 170.0, 320.0,  8.0,  5.0), // E vs A
        array( 95.0, 150.0, 290.0,  8.0,  5.0), // E vs B
        array( 75.0, 120.0, 250.0,  8.0,  5.0), // E vs C
        array( 55.0,  90.0, 200.0,  8.0,  4.0), // E vs D
        array( 40.0,  75.0, 190.0,  9.0, -4.0), // E vs E
        array( 38.0,  65.0, 160.0,  8.0, -6.0), // E vs F
        array( 38.0,  65.0, 160.0,  8.0, -6.0), // E vs G
        array( 38.0,  65.0, 160.0,  8.0, -6.0), // E vs H
    ),
    // F
    array(
        array(150.0, 250.0, 400.0,  6.0,  3.0), // F vs A
        array(130.0, 220.0, 370.0,  6.0,  3.0), // F vs B
        array(110.0, 190.0, 340.0,  6.0,  3.0), // F vs C
        array( 85.0, 150.0, 300.0,  6.0,  3.0), // F vs D
        array( 60.0, 110.0, 260.0,  7.0,  4.0), // F vs E
        array( 34.0,  90.0, 240.0,  7.0, -2.0), // F vs F
        array( 34.0,  70.0, 180.0,  6.0, -3.0), // F vs G
        array( 34.0,  70.0, 180.0,  6.0, -3.0), // F vs H
    ),
    // G
    array(
        array( 34.0,  50.0, 500.0,  8.0,  9.0), // G vs A
        array( 34.0,  50.0, 450.0,  7.0,  7.0), // G vs B
        array( 34.0,  50.0, 400.0,  7.0,  5.0), // G vs C
        array( 34.0,  50.0, 350.0,  7.0,  3.0), // G vs D
        array( 34.0,  55.0, 300.0,  7.0,  2.0), // G vs E
        array( 34.0,  55.0, 250.0,  6.0,  1.0), // G vs F
        array( 34.0,  60.0, 200.0,  7.0, -3.0), // G vs G
        array( 34.0,  55.0, 180.0,  6.0, -2.0), // G vs H
    ),
    // H
    array(
        array( 90.0, 140.0, 380.0,  5.0,  2.0), // H vs A
        array( 80.0, 130.0, 350.0,  5.0,  2.0), // H vs B
        array( 70.0, 120.0, 320.0,  5.0,  2.0), // H vs C
        array( 60.0, 110.0, 290.0,  5.0,  2.0), // H vs D
        array( 55.0, 100.0, 260.0,  5.0,  2.0), // H vs E
        array( 50.0,  90.0, 230.0,  5.0,  1.0), // H vs F
        array( 45.0,  80.0, 200.0,  5.0,  1.0), // H vs G
        array( 34.0,  70.0, 180.0,  5.0, -1.0), // H vs H
    ),
);


const COLORS: array<vec4<f32>, 8> = array(
    vec4(0.85, 0.05, 0.85, 1.0),
    vec4(0.05, 0.05, 0.90, 1.0),
    vec4(0.05, 0.80, 0.20, 1.0),
    vec4(0.90, 0.45, 0.05, 1.0),
    vec4(0.05, 0.75, 0.75, 1.0),
    vec4(0.90, 0.90, 0.05, 1.0),
    vec4(0.60, 0.05, 0.90, 1.0),
    vec4(0.85, 0.15, 0.15, 1.0),
);

