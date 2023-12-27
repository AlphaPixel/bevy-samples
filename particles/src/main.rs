use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::*;
use std::time::{Duration, Instant};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

// FPS counter module
mod fps;
use fps::{fps_counter_showhide, fps_text_update_system, setup_fps_counter};

// Compile time constants
const PARTICLE_RADIUS: f32 = 0.2;
const SPAWN_COUNT: usize = 30; // Number of particles to spawn when it's time to do so.
const PARTICLE_EXPIRE_TIME_SECS: u64 = 10; // Number of seconds until each particle despawns.
const PARTICLE_RESPAWN_TIME_MS: u64 = 100; // How often (in milliseconds) to wait until spawning more particles.
const MAX_SPAWN_OFFSET: f32 = 3.0; // Max offset (in X, Z) of new particle location.
const INITIAL_VELOCITY: f32 = 2.0; // Initial velocity vector magnitude of new particles.
const GROUND_RADIUS: f32 = 10.0; // The "radius" of the ground plane.

fn main() {
    // Create the bevy 'app' and add all of the plugins/systems.
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin {})
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_particles, despawn_particles))
        .add_systems(Update, bevy::window::close_on_esc)
        // FPS display
        .add_systems(Startup, setup_fps_counter)
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        //
        .run();
}

// ParticleMarker - this component marks an entity as a particle.  Used for querying inside systems.
#[derive(Component)]
struct ParticleMarker;

// ExpireTime - a component that denotes the time an entity should live before despawning.
#[derive(Component)]
struct ExpireTime(Instant);
impl Default for ExpireTime {
    fn default() -> Self {
        ExpireTime(Instant::now())
    }
}

// Configuration - global resource containing system wide data.
#[derive(Resource)]
struct Configuration {
    // The mesh for the particle.  Created once at setup and reused for all subsequent particles.
    sphere_mesh: Handle<Mesh>,
    // The material for the particle.  Created once at setup and reused for all subsequent particles.
    particle_material: Handle<StandardMaterial>,
    // Used to determine how much time should elapse before spawning new particles.
    spawn_delta: Duration,
}

// Particle - A bundle (bevy-speak) containing the components that define a particle.
#[derive(Bundle)]
struct Particle {
    // When should this particle expire (despawn)
    expire_time: ExpireTime,
    // Marker denoting this entity is a particle
    marker: ParticleMarker,
    // Particle's velocity vector
    velocity: Velocity,
    // Particles geometry
    geometry: PbrBundle,
}

// setup - a setup system that creates global data and spawns fixed/static entities (camera, lights, ground, etc.)
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the material the particles will use (this will be added to the configuration
    // resource for later use)
    let particle_material = materials.add(StandardMaterial {
        base_color: Color::hex("#ff6060").unwrap(),
        metallic: 1.0,
        perceptual_roughness: 0.5,
        ..default()
    });

    // Create the mesh the particles will use (this will be added to the configuration resource
    // for later use)
    let sphere_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: PARTICLE_RADIUS,
            ..default()
        })
        .unwrap(),
    );

    // Add the configuration resource to the world.
    commands.insert_resource(Configuration {
        sphere_mesh,
        particle_material,
        spawn_delta: Duration::from_millis(PARTICLE_RESPAWN_TIME_MS),
    });

    // Create the ground entity
    {
        // Define the ground's boundary.  The will be given to the
        // ground's collider so it interacts with the physics engine)
        let ground_boundary = &[
            Vec3::new(GROUND_RADIUS, 0., GROUND_RADIUS),
            Vec3::new(GROUND_RADIUS, 0., -GROUND_RADIUS),
            Vec3::new(-GROUND_RADIUS, 0., -GROUND_RADIUS),
            Vec3::new(-GROUND_RADIUS, 0., GROUND_RADIUS),
            Vec3::new(GROUND_RADIUS, -10.0, GROUND_RADIUS),
            Vec3::new(GROUND_RADIUS, -10.0, -GROUND_RADIUS),
            Vec3::new(-GROUND_RADIUS, -10.0, -GROUND_RADIUS),
            Vec3::new(-GROUND_RADIUS, -10.0, GROUND_RADIUS),
        ];

        // Spawn the ground plane - then insert the physics type and collider.
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane {
                    size: GROUND_RADIUS * 2.0,
                    subdivisions: 16,
                })),
                material: materials.add(Color::rgb(0.4, 0.4, 0.4).into()),
                transform: Transform::from_translation(Vec3::Y / 2.0),
                ..Default::default()
            })
            .insert(RigidBody::Fixed)
            .insert(Collider::convex_hull(ground_boundary).unwrap());
    }

    // Spawn a simple point light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 0.0),
        point_light: PointLight {
            intensity: 600000.,
            range: 500.,
            ..default()
        },
        ..default()
    });

    // Spawn a simple perspective camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20.0, 20.0, 20.0).looking_at(Vec3::default(), Vec3::Y),
        projection: PerspectiveProjection { ..default() }.into(),
        ..default()
    });
}

// spawn_particle - an 'update' system that spawns new particles if it's time to do so.
fn spawn_particles(
    configuration: Res<Configuration>,
    mut next_spawn_deadline: Local<ExpireTime>,
    mut commands: Commands,
) {
    // If it's time to spawn more particles...
    if Instant::now() > next_spawn_deadline.0 {
        // Spawn 'SPAWN_COUNT' particles
        for _ in 0..SPAWN_COUNT {
            // Create three random vector components that will be the initial velocity
            // vector of the new particle
            let x = ((random::<f32>() * 2.0) - 1.0) * 0.25;
            let y = 1.0;
            let z = ((random::<f32>() * 2.0) - 1.0) * 0.25;

            // Create the initial velocity vector
            let v = Vec3::new(x, y, z).normalize() * INITIAL_VELOCITY;

            // Create a random vector that will contain the initial starting position
            // of the particle.
            let x = 1.0 + random::<f32>() * 2.0;
            let y = MAX_SPAWN_OFFSET + 1.0 + random::<f32>() * 1.0;
            let z = 1.0 + random::<f32>() * 2.0;

            // Spawn the particle using our Particle bundle struct.
            commands
                .spawn(Particle {
                    expire_time: ExpireTime(
                        Instant::now() + Duration::from_secs(PARTICLE_EXPIRE_TIME_SECS),
                    ),
                    marker: ParticleMarker {},
                    velocity: Velocity {
                        linvel: v,
                        angvel: Vec3::ZERO,
                    },

                    // Set up the PBR bundle for the geometry that represents the particle (a simple sphere)
                    geometry: PbrBundle {
                        mesh: configuration.sphere_mesh.clone(),
                        transform: Transform::from_translation(Vec3::new(x, y, z)),
                        material: configuration.particle_material.clone(),
                        ..default()
                    },
                })
                // Insert a dynamic rigid body component for the particle
                .insert(RigidBody::Dynamic)
                // Insert a collider component for the particle
                .insert(Collider::ball(PARTICLE_RADIUS));
        }

        // Udpate the deadline for the next round of particle spawns.
        *next_spawn_deadline = ExpireTime(Instant::now() + configuration.spawn_delta);
    }
}

// despawn_particles - an update system that will despawn any particles that have outlived
// their expire-time.
fn despawn_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &ExpireTime), With<ParticleMarker>>,
) {
    // Determine if it's time to despawn particles...if so, do it.
    let now = Instant::now();
    for (entity, expire_time) in query.iter_mut() {
        if now >= expire_time.0 {
            commands.entity(entity).despawn()
        }
    }
}
