use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::ShaderType}};
use rand::{self, RngExt};

use crate::simulation::{self, critter_color};



// ========== LEGACY ========== 
#[derive(Clone, Copy, ShaderType)]
pub struct ShaderCritter
{
    pub center: Vec2,
    pub color: LinearRgba,
    pub radius: f32,
    pub _padding: Vec3
} 

#[derive(Component)] 
pub struct PhysicsCritter 
{ 
    pub color: LinearRgba,
    pub radius: f32,
    pub velocity: Vec2
}
// ========== LEGACY ========== 


#[derive(Resource, Clone, ExtractResource, ShaderType)]
pub struct CritterUniforms
{
    pub dt: f32,
    pub damping: f32,
    pub critter_count: u32
}

#[derive(Clone, Copy, ShaderType)]
pub struct GpuCritter
{
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub color: u32,
    pub _padding: Vec2
}


pub fn initial_critters(
    mut commands: Commands
)
{

    let mut rng = rand::rng();
    
    for _ in 0..850
    {

        let cx = rand::random::<f32>() * 1000.0;
        let cy = rand::random::<f32>() * 1000.0;
          
        let random = rng.random_range(0..8);
        let color: LinearRgba; 

        match random
        {
            0 => {color = critter_color::A;},
            1 => {color = critter_color::B;},
            2 => {color = critter_color::C;},
            3 => {color = critter_color::D;},
            4 => {color = critter_color::E;},
            5 => {color = critter_color::F;},
            6 => {color = critter_color::G;},
            7 => {color = critter_color::H;},
            _ => {color = critter_color::H;}
        }

        commands.spawn(( 
            PhysicsCritter
            {
                color: color, 
                radius: rand::random_range(8.0..20.0),
                velocity: Vec2::ZERO
            },
            Transform::from_xyz(cx, cy, 0.0)
        )); // spawn
        
    } // for i
}


pub fn critters_velocity(
    mut critters: Query<(Entity, &mut PhysicsCritter, &Transform)>,
    time: Res<Time>,
) {
    // snapshot: just positions and colors, keyed by entity
    let snapshot: Vec<(Entity, LinearRgba, Vec2, f32)> = critters
        .iter()
        .map(|(e, c, t)| (e, c.color, t.translation.truncate(), c.radius))
        .collect();

    // mutate velocity of each critter based on all others
    for (entity, mut critter, transform) in &mut critters {
        let my_pos = transform.translation.truncate();
        let rad = critter.radius.clone();

        for (other_entity, other_color, other_pos, other_rad) in &snapshot {
            if *other_entity == entity { continue; } // skip self

            let to_other = *other_pos - my_pos;
            let dist = to_other.length();
            if dist < rad / 4.0 { continue; }

            let combined_rad = rad + other_rad;
            let dir = to_other.normalize();

            // Standard Behaviour

            let stats = match (critter.color, *other_color) {
                // A: dense nucleus — rigid, pulls B in, ejects everything else
                (critter_color::A, critter_color::A) => (34.0,  36.0,  80.0, 16.0, 5.0),
                (critter_color::A, critter_color::B) => (34.0,  38.0, 130.0, 13.0, 2.0),
                (critter_color::A, critter_color::C) => (34.0,  36.0,  55.0, 15.0,-5.0),
                (critter_color::A, critter_color::D) => (34.0,  36.0,  55.0, 15.0,-5.0),
                (critter_color::A, critter_color::E) => (34.0,  36.0,  55.0, 15.0,-5.0),
                (critter_color::A, critter_color::F) => (34.0,  36.0,  55.0, 15.0,-5.0),
                (critter_color::A, critter_color::G) => (34.0,  36.0,  55.0, 15.0,-5.0),
                (critter_color::A, critter_color::H) => (34.0,  36.0,  55.0, 15.0,-5.0),

                // B: inner cytoplasm — orbits A tightly, pulls C inward, repels outer layers
                (critter_color::B, critter_color::A) => (40.0,  55.0, 130.0, 13.0, 2.0),
                (critter_color::B, critter_color::B) => (34.0,  40.0, 100.0, 12.0, 1.0),
                (critter_color::B, critter_color::C) => (34.0,  42.0,  90.0, 11.0,  8.0),
                (critter_color::B, critter_color::D) => (34.0,  38.0,  65.0, 12.0,-1.0),
                (critter_color::B, critter_color::E) => (34.0,  38.0,  65.0, 12.0,-1.0),
                (critter_color::B, critter_color::F) => (34.0,  38.0,  65.0, 12.0,-1.0),
                (critter_color::B, critter_color::G) => (34.0,  38.0,  65.0, 12.0,-1.0),
                (critter_color::B, critter_color::H) => (34.0,  38.0,  65.0, 12.0,-1.0),

                // C: outer cytoplasm — loose ring around B, bridges inner and outer layers
                (critter_color::C, critter_color::A) => (55.0,  90.0, 180.0, 10.0,  0.6),
                (critter_color::C, critter_color::B) => (45.0,  65.0, 150.0, 10.0,  0.9),
                (critter_color::C, critter_color::C) => (34.0,  44.0, 120.0, 10.0,  0.7),
                (critter_color::C, critter_color::D) => (34.0,  42.0,  90.0, 10.0,  0.5),
                (critter_color::C, critter_color::E) => (34.0,  42.0,  80.0,  9.0, -0.6),
                (critter_color::C, critter_color::F) => (34.0,  42.0,  80.0,  9.0, -0.6),
                (critter_color::C, critter_color::G) => (34.0,  42.0,  80.0,  9.0, -0.6),
                (critter_color::C, critter_color::H) => (34.0,  42.0,  80.0,  9.0, -0.8),

                // D: inner membrane — forms a defined shell, repels inward and outward equally
                (critter_color::D, critter_color::A) => (80.0, 130.0, 260.0,  9.0,  0.6),
                (critter_color::D, critter_color::B) => (70.0, 110.0, 220.0,  9.0,  0.7),
                (critter_color::D, critter_color::C) => (55.0,  85.0, 180.0,  9.0,  0.6),
                (critter_color::D, critter_color::D) => (45.0,  65.0, 180.0, 10.0, -0.5),
                (critter_color::D, critter_color::E) => (40.0,  60.0, 140.0,  9.0, -0.8),
                (critter_color::D, critter_color::F) => (40.0,  60.0, 140.0,  9.0, -0.8),
                (critter_color::D, critter_color::G) => (40.0,  60.0, 140.0,  9.0, -0.8),
                (critter_color::D, critter_color::H) => (40.0,  60.0, 140.0,  9.0, -0.8),

                // E: outer membrane — spreads wide, defines the cell boundary
                (critter_color::E, critter_color::A) => (110.0, 170.0, 320.0,  8.0,  0.5),
                (critter_color::E, critter_color::B) => (95.0,  150.0, 290.0,  8.0,  0.5),
                (critter_color::E, critter_color::C) => (75.0,  120.0, 250.0,  8.0,  0.5),
                (critter_color::E, critter_color::D) => (55.0,   90.0, 200.0,  8.0,  0.4),
                (critter_color::E, critter_color::E) => (40.0,   75.0, 190.0,  9.0, -0.4),
                (critter_color::E, critter_color::F) => (38.0,   65.0, 160.0,  8.0, -0.6),
                (critter_color::E, critter_color::G) => (38.0,   65.0, 160.0,  8.0, -0.6),
                (critter_color::E, critter_color::H) => (38.0,   65.0, 160.0,  8.0, -0.6),

                // F: flagella — long range, loosely tethered to E, trails behind the creature
                (critter_color::F, critter_color::A) => (150.0, 250.0, 400.0,  6.0,  0.3),
                (critter_color::F, critter_color::B) => (130.0, 220.0, 370.0,  6.0,  0.3),
                (critter_color::F, critter_color::C) => (110.0, 190.0, 340.0,  6.0,  0.3),
                (critter_color::F, critter_color::D) => (85.0,  150.0, 300.0,  6.0,  0.3),
                (critter_color::F, critter_color::E) => (60.0,  110.0, 260.0,  7.0,  0.4),
                (critter_color::F, critter_color::F) => (34.0,   90.0, 240.0,  7.0, -0.2),
                (critter_color::F, critter_color::G) => (34.0,   70.0, 180.0,  6.0, -0.3),
                (critter_color::F, critter_color::H) => (34.0,   70.0, 180.0,  6.0, -0.3),

                // G: scouts — highly repelled by core, attracted to other creatures' cores from far away
                // creates the chasing behavior between creatures
                (critter_color::G, critter_color::A) => (34.0,   50.0, 500.0,  8.0,  0.9),
                (critter_color::G, critter_color::B) => (34.0,   50.0, 450.0,  7.0,  0.7),
                (critter_color::G, critter_color::C) => (34.0,   50.0, 400.0,  7.0,  0.5),
                (critter_color::G, critter_color::D) => (34.0,   50.0, 350.0,  7.0,  0.3),
                (critter_color::G, critter_color::E) => (34.0,   55.0, 300.0,  7.0,  0.2),
                (critter_color::G, critter_color::F) => (34.0,   55.0, 250.0,  6.0,  0.1),
                (critter_color::G, critter_color::G) => (34.0,   60.0, 200.0,  7.0, -0.3),
                (critter_color::G, critter_color::H) => (34.0,   55.0, 180.0,  6.0, -0.2),

                // H: extracellular fluid — very loosely bound, fills space between creatures
                // slightly attracted to everything at long range, repels at close range
                (critter_color::H, critter_color::A) => (90.0,  140.0, 380.0,  5.0,  0.2),
                (critter_color::H, critter_color::B) => (80.0,  130.0, 350.0,  5.0,  0.2),
                (critter_color::H, critter_color::C) => (70.0,  120.0, 320.0,  5.0,  0.2),
                (critter_color::H, critter_color::D) => (60.0,  110.0, 290.0,  5.0,  0.2),
                (critter_color::H, critter_color::E) => (55.0,  100.0, 260.0,  5.0,  0.2),
                (critter_color::H, critter_color::F) => (50.0,   90.0, 230.0,  5.0,  0.1),
                (critter_color::H, critter_color::G) => (45.0,   80.0, 200.0,  5.0,  0.1),
                (critter_color::H, critter_color::H) => (34.0,   70.0, 180.0,  5.0, -0.1),

                _ => (0.0, 0.0, 0.0, 0.0, 0.0)
            };

            if dist < rad / 1.2
            {
                critter.velocity -= dir * time.delta_secs();
            }
            else
            {
                critter.velocity += dir * calc_critter_force(dist, stats) * time.delta_secs() * 2.0;
            }

        }
    }
}

// (repel_radius, orbit_radius, perception_radius, repel_force, attract_force)
fn calc_critter_force(dist: f32, stats: (f32, f32, f32, f32, f32)) -> f32 {
    let (repel_radius, orbit_radius, perception_radius, repel_force, attract_force) = stats;

    if dist > perception_radius { return 0.0; }

    // zone 1: too close — hard repel
    if dist < repel_radius {
        let t = 1.0 - (dist / repel_radius);
        return -repel_force * t; 
    }

    // zone 2: between repel and orbit — attract toward orbit
    if dist < orbit_radius {
        let t = (dist - repel_radius) / (orbit_radius - repel_radius);
        return attract_force * (1.0 - t); // pull inward
    }

    // zone 3: between orbit and perception — attract back toward orbit
    let t = (dist - orbit_radius) / (perception_radius - orbit_radius);
    return attract_force * (1.0 - t); // weaker pull as they get farther
}

pub fn move_critters(
    world: Res<simulation::world::World>,
    critters: Query<(&mut PhysicsCritter, &mut Transform)>,
    time: Res<Time>,
)
{

    for (mut critter, mut tfrm) in critters
    {


        let critter_vel = critter.velocity.clone();

        tfrm.translation += critter_vel.extend(0.0) * time.delta_secs() * 20.0;
        critter.velocity -= critter_vel * world.drag_coefficient; 
    }

}
