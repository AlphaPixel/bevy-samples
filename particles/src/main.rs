use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::*;
use std::time::{Duration, Instant};

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

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugins(RapierDebugRenderPlugin::default())
    .add_systems(Startup, setup)
    .add_systems(Update, (spawn_particles, update_particles))
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the mesh the particles will use
    let sphere_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.25,
            ..default()
        })
        .unwrap(),
    );

    commands.insert_resource(Configuration {
        sphere_mesh,
        spawn_delta: Duration::from_millis(100),
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
    if Instant::now() > next_spawn_deadline.0 {

        for _ in 0..20 {
            let x = ((random::<f32>() * 2.0) - 1.0) * 0.25;
            let y = 1.0;
            let z = ((random::<f32>() * 2.0) - 1.0) * 0.25;

            let v = Vec3::new(x, y, z).normalize() * 9.81;

            let x = 1.0 + random::<f32>() * 2.0;
            let y = 1.0 + random::<f32>() * 1.0;
            let z = 1.0 + random::<f32>() * 2.0;

            commands.spawn(Particle {
                expire_time: ExpireTime(Instant::now() + Duration::from_secs(20)),
                marker: ParticleMarker {},
                velocity: Velocity {
                    linvel: v,
                    angvel: Vec3::ZERO,
                },

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
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(0.25));
        }

        *next_spawn_deadline = ExpireTime(Instant::now() + configuration.spawn_delta);
    }
}

fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &ExpireTime, &ParticleMarker)>
) {
    let now = Instant::now();

    for (entity, expire_time, _) in query.iter_mut() {
        if now >= expire_time.0 {
            commands.entity(entity).despawn()
        }
    }
}
