use bevy::prelude::*;
use rand::*;
use std::time::{Duration, Instant};

#[derive(Component)]
struct ParticleMarker;

#[derive(Component)]
struct Velocity(Vec3);


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
    .add_systems(Startup, setup)
    .add_systems(Update, (spawn_particles, update_particles))
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create the mesh the particles will use
    let sphere_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 1.0,
            ..default()
        })
        .unwrap(),
    );

    commands.insert_resource(Configuration {
        sphere_mesh,
        spawn_delta: Duration::from_millis(1),
    });

    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0),
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
            transform: Transform::from_xyz(80.0, 80.0, 80.0).looking_at(Vec3::default(), Vec3::Y),
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

            let v = Vec3::new(x, y, z).normalize() * 5.0;

            commands.spawn(Particle {
                expire_time: ExpireTime(Instant::now() + Duration::from_secs(5)),
                marker: ParticleMarker {},
                velocity: Velocity(v),

                geometry: PbrBundle {
                    mesh: configuration.sphere_mesh.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::hex("#ffd891").unwrap(),
                        metallic: 1.0,
                        perceptual_roughness: 0.5,
                        ..default()
                    }),
                    ..default()
                }
            });
        }

        *next_spawn_deadline = ExpireTime(Instant::now() + configuration.spawn_delta);
    }
}

fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &ExpireTime, &ParticleMarker)>
) {
    let now = Instant::now();

    for (entity, mut transform, mut velocity, expire_time, _) in query.iter_mut() {
        if now >= expire_time.0 {
            commands.entity(entity).despawn()
        } else {
            let v = velocity.0 + Vec3::new(0., -30. * time.delta_seconds(), 0.);

            *velocity = Velocity(v);

            if transform.translation.y + v.y < 0.0 {
                transform.translation.y = 0.;
                transform.translation.x += v.x;
                transform.translation.z += v.z;

                velocity.0.x *= 0.98;
                velocity.0.y = -velocity.0.y * 0.4;
                velocity.0.z *= 0.98;
            } else {
                transform.translation += v;
            }
        }
    }
}
