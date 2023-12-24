use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::*;
use std::time::{Duration, Instant};

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
//    .add_plugins(RapierDebugRenderPlugin::default())
    .add_systems(Startup, setup)
    .add_systems(Update, (spawn_particles, update_particles))
    .add_systems(Startup, setup_fps_counter)
    .add_systems(Update, (
        fps_text_update_system,
        fps_counter_showhide,
    ))
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

    // let cube_boundary = &[
    //     Vec3::new(CUBE_SIZE, 0., CUBE_SIZE),
    //     Vec3::new(CUBE_SIZE, 0., -CUBE_SIZE),
    //     Vec3::new(-CUBE_SIZE, 0., -CUBE_SIZE),
    //     Vec3::new(-CUBE_SIZE, 0., CUBE_SIZE),

    //     Vec3::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
    //     Vec3::new(CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE),
    //     Vec3::new(-CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE),
    //     Vec3::new(-CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
    // ];

    // commands
    // .spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube {
    //         size: CUBE_SIZE,
    //     })),
    //     material: materials.add(Color::rgb(0.7, 0.3, 0.4).into()),
    //     transform: Transform::from_translation(Vec3::new(6., 6., 6.)),
    //     ..Default::default()
    // })
    // .insert(RigidBody::Fixed)
    // .insert(Collider::convex_hull(cube_boundary).unwrap());

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

        for _ in 0..SPAWN_COUNT {
            let x = ((random::<f32>() * 2.0) - 1.0) * 0.25;
            let y = 1.0;
            let z = ((random::<f32>() * 2.0) - 1.0) * 0.25;

            let v = Vec3::new(x, y, z).normalize() * INITIAL_VELOCITY;

            let x = 1.0 + random::<f32>() * 2.0;
            let y = CUBE_SIZE + 1.0 + random::<f32>() * 1.0;
            let z = 1.0 + random::<f32>() * 2.0;

            commands.spawn(Particle {
                expire_time: ExpireTime(Instant::now() + Duration::from_secs(PARTICLE_EXPIRE_TIME_SECS)),
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
            .insert(Collider::ball(RADIUS));
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



use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

/// Marker to find the container entity so we can show/hide the FPS counter
#[derive(Component)]
struct FpsRoot;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

fn setup_fps_counter(
    mut commands: Commands,
) {
    // create our UI root node
    // this is the wrapper/container for the text
    let root = commands.spawn((
        FpsRoot,
        NodeBundle {
            // give it a dark background for readability
            background_color: BackgroundColor(Color::BLACK.with_a(0.5)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                position_type: PositionType::Absolute,
                // position it at the top-right corner
                // 1% away from the top window edge
                right: Val::Percent(1.),
                top: Val::Percent(1.),
                // set bottom/left to Auto, so it can be
                // automatically sized depending on the text
                bottom: Val::Auto,
                left: Val::Auto,
                // give it some padding for readability
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    // create our text
    let text_fps = commands.spawn((
        FpsText,
        TextBundle {
            // use two sections, so it is easy to update just the number
            text: Text::from_sections([
                TextSection {
                    value: "FPS: ".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        // if you want to use your game's font asset,
                        // uncomment this and provide the handle:
                        // font: my_font_handle
                        ..default()
                    }
                },
                TextSection {
                    value: " N/A".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        // if you want to use your game's font asset,
                        // uncomment this and provide the handle:
                        // font: my_font_handle
                        ..default()
                    }
                },
            ]),
            ..Default::default()
        },
    )).id();
    commands.entity(root).push_children(&[text_fps]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy
        if let Some(value) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            // Format the number as to leave space for 4 digits, just in case,
            // right-aligned and rounded. This helps readability when the
            // number changes rapidly.
            text.sections[1].value = format!("{value:>4.0}");

            // Let's make it extra fancy by changing the color of the
            // text according to the FPS value:
            text.sections[1].style.color = if value >= 120.0 {
                // Above 120 FPS, use green color
                Color::rgb(0.0, 1.0, 0.0)
            } else if value >= 60.0 {
                // Between 60-120 FPS, gradually transition from yellow to green
                Color::rgb(
                    (1.0 - (value - 60.0) / (120.0 - 60.0)) as f32,
                    1.0,
                    0.0,
                )
            } else if value >= 30.0 {
                // Between 30-60 FPS, gradually transition from red to yellow
                Color::rgb(
                    1.0,
                    ((value - 30.0) / (60.0 - 30.0)) as f32,
                    0.0,
                )
            } else {
                // Below 30 FPS, use red color
                Color::rgb(1.0, 0.0, 0.0)
            }
        } else {
            // display "N/A" if we can't get a FPS measurement
            // add an extra space to preserve alignment
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::WHITE;
        }
    }
}

/// Toggle the FPS counter when pressing F12
fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<Input<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}