use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::*;
use std::time::{Duration, Instant};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

mod fps;
use fps::{setup_fps_counter, fps_text_update_system, fps_counter_showhide};

const RADIUS: f32 = 0.2;
const SPAWN_COUNT: usize = 30;
const PARTICLE_EXPIRE_TIME_SECS: u64 = 10;
const PARTICLE_RESPAWN_TIME_MS: u64 = 100;
const CUBE_SIZE: f32 = 3.0;
const INITIAL_VELOCITY: f32 = 1.81;

fn main() {
    App::new()
    .add_plugins(FrameTimeDiagnosticsPlugin::default())    
    .add_plugins(DefaultPlugins)
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    .add_systems(Startup, setup)
    .add_systems(Update, (spawn_particles, update_particles))
    .add_systems(Update, bevy::window::close_on_esc)
    // FPS display
    .add_systems(Startup, setup_fps_counter)
    .add_systems(Update, (
        fps_text_update_system,
        fps_counter_showhide,
    ))
    //
    .run();
}

#[derive(Component)]
struct ParticleMarker;

#[derive(Component)]
struct ExpireTime(Instant);

impl Default for ExpireTime {
    fn default() -> Self {
        ExpireTime(Instant::now())
    }
}

#[derive(Resource)]
struct Configuration {
    sphere_mesh: Handle<Mesh>,
    spawn_delta: Duration,
}

#[derive(Bundle)]
struct Particle {
    expire_time: ExpireTime,
    marker: ParticleMarker,

    velocity: Velocity,

    geometry: PbrBundle,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the mesh the particles will use
    let sphere_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: RADIUS,
            ..default()
        })
        .unwrap(),
    );

    commands.insert_resource(Configuration {
        sphere_mesh,
        spawn_delta: Duration::from_millis(PARTICLE_RESPAWN_TIME_MS),
    });

    // ground
    let ground_boundary = &[
        Vec3::new(128.0, 0., 128.0),
        Vec3::new(128.0, 0., -128.0),
        Vec3::new(-128.0, 0., -128.0),
        Vec3::new(-128.0, 0., 128.0),

        Vec3::new(128.0, -10.0, 128.0),
        Vec3::new(128.0, -10.0, -128.0),
        Vec3::new(-128.0, -10.0, -128.0),
        Vec3::new(-128.0, -10.0, 128.0),
    ];

    commands
    // Spawn the ground plane
    .spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { 
            size: 128.0,
            subdivisions: 16,
         }
        )),
        material: materials.add(Color::rgb(0.4, 0.4, 0.4).into()),
        transform: Transform::from_translation(Vec3::Y / 2.0),
        ..Default::default()
    })
    .insert(RigidBody::Fixed)
    .insert(Collider::convex_hull(ground_boundary).unwrap());

    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 0.0),
        point_light: PointLight {
            intensity: 600000.,
            range: 500.,
            ..default()
        },
        ..default()
    });

    // camera
    commands.spawn(
        Camera3dBundle {
            transform: Transform::from_xyz(20.0, 20.0, 20.0).looking_at(Vec3::default(), Vec3::Y),
            projection: PerspectiveProjection {
                ..default()
            }
            .into(),
            ..default()
        }
    );
}

fn spawn_particles(
    configuration: Res<Configuration>,
    mut next_spawn_deadline: Local<ExpireTime>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            let y = CUBE_SIZE + 1.0 + random::<f32>() * 1.0;
            let z = 1.0 + random::<f32>() * 2.0;

            // Spawn the particle using our Particle bundle struct.
            commands.spawn(Particle {
                expire_time: ExpireTime(Instant::now() + Duration::from_secs(PARTICLE_EXPIRE_TIME_SECS)),
                marker: ParticleMarker {},
                velocity: Velocity {
                    linvel: v,
                    angvel: Vec3::ZERO,
                },

                // Set up the PBR bundle for the geometry that represents the particle (a simple sphere)
                geometry: PbrBundle {
                    mesh: configuration.sphere_mesh.clone(),
                    transform: Transform::from_translation(Vec3::new(x, y, z)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::hex("#ffd891").unwrap(),
                        metallic: 1.0,
                        perceptual_roughness: 0.5,
                        ..default()
                    }),
                    ..default()
                }
            })
            // Insert a dynamic rigid body component for the particle
            .insert(RigidBody::Dynamic)
            // Insert a collider component for the particle
            .insert(Collider::ball(RADIUS));
        }

        // Udpate the deadline for the next round of particle spawns.
        *next_spawn_deadline = ExpireTime(Instant::now() + configuration.spawn_delta);
    }
}

fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &ExpireTime, &ParticleMarker)>
) {
    // Determine if it's time to despawn particles...if so, do it.
    let now = Instant::now();
    for (entity, expire_time, _) in query.iter_mut() {
        if now >= expire_time.0 {
            commands.entity(entity).despawn()
        }
    }
}
