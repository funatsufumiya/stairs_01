use std::f32::consts::PI;

use bevy::{gltf::{GltfMesh, GltfNode}, math::ops::sin_cos, prelude::*};

use bevy_asset_loader::asset_collection::AssetCollection;

#[cfg(feature = "egui")]
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bimap::BiMap;
use rand::{rngs::StdRng, Rng as _, SeedableRng};

fn main() {
    use bevy_asset_loader::loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt};

    let mut app = App::new();

    app
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(AmbientLight {
            // brightness: 750.0,
            brightness: 200.0,
            ..default()
        })
        .insert_resource(StairParam {
            count: 15,
            height: 1.95,
            depth: 1.95,
            span_sec: 1.0,
            last_stair_pos: None,
        })
        .init_state::<AssetLoadingState>()
        .add_loading_state(
            LoadingState::new(AssetLoadingState::Loading)
                .continue_to_state(AssetLoadingState::Loaded)
                .load_collection::<GltfAssets>()
        )
        .add_systems(Startup, spawn_loading_text)
        .add_systems(OnEnter(AssetLoadingState::Loaded), cleanup_loading_text.before(setup))
        .add_systems(OnEnter(AssetLoadingState::Loaded), setup)
        .add_systems(Update, move_stairs)
        .add_systems(Update, swing_camera)
        ;

    #[cfg(feature = "egui")]
    app
        .add_plugins(EguiPlugin)
        .add_systems(Update, ui_system);

    app
        .run();
}

#[derive(Component)]
struct LoadingText;

#[derive(Component)]
struct Stair {
    pub init_pos: Vec3,
}

#[derive(Resource)]
struct StairParam {
    pub count: usize,
    pub height: f32,
    pub depth: f32,
    pub span_sec: f32,
    pub last_stair_pos: Option<Vec3>,
}

fn spawn_loading_text(mut commands: Commands) {
    commands
        .spawn( (
            Text::new("loading..."),
            Node {
                position_type: PositionType::Relative,
                top: Val::Percent(50.0),
                left: Val::Percent(50.0),
                ..default()
            },
            LoadingText,
        ));
}

fn cleanup_loading_text(
    mut commands: Commands,
    loading_text: Query<Entity, With<LoadingText>>,
) {
    for entity in loading_text.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
  #[asset(path = "models/stairs.glb")]
  pub iroha: Handle<Gltf>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum AssetLoadingState {
    #[default]
    Loading,
    Loaded,
}

fn setup(
    mut commands: Commands,
    // mut asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltf_res: Res<GltfAssets>,
    assets_gltf: Res<Assets<Gltf>>,
    // assets_gltfmeshes: Res<Assets<GltfMesh>>,
    // assets_gltfnodes: Res<Assets<GltfNode>>,
    mut stair_param: ResMut<StairParam>,
    // mut meshes: ResMut<Assets<Mesh>>,
) {
    // Create a camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-4.0, 1.5, 5.0)
            .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 300_000.0, // lumens
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 7.0, 4.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    let gltf = assets_gltf.get(&gltf_res.iroha).unwrap();
    // let scene = GltfAssetLabel::Scene(0).from_asset("models/iroha.glb");
    let scene = gltf.scenes[0].clone();

    for i in 0..stair_param.count {
        let x = 0 as f32;
        let y = (stair_param.height as f32) * ((i as f32) - 2.0);
        let z = - (stair_param.depth as f32) * ((i as f32) - 2.0);
        commands.spawn((
            SceneRoot(scene.clone()),
            Transform::from_xyz(x, y, z),
            MeshMaterial3d( materials.add(
                StandardMaterial {
                    base_color: Color::srgb(0.8, 0.7, 0.6),
                    metallic: 0.9,
                    ..default()
                }
            )),
            Stair {
                init_pos: Vec3::new(x, y, z),
            }
        ));

        // if i == stair_param.count - 1 {
        if i == 6 {
            stair_param.last_stair_pos = Some(Vec3::new(x, y, z));
        }
    }

        // commands.spawn((
        //     Mesh3d(mesh_handle.clone()),
        //     Transform::from_xyz(x, y, z).with_rotation(rotation),
        //     MeshMaterial3d( materials.add(
        //         StandardMaterial {
        //             base_color: Color::srgb(0.8, 0.7, 0.6),
        //             ..default()
        //         }
        //     ))
        // ));
}

// fn rotate_mesh(
//     time: Res<Time>,
//     mut meshes: Query<&mut Transform, With<Mesh3d>>,
// ) {
//     for mut transform in meshes.iter_mut() {
//         transform.rotate_x(time.delta_secs());
//         transform.rotate_y(time.delta_secs());
//     }
// }

fn move_stairs(
    time: Res<Time>,
    mut stair_param: ResMut<StairParam>,
    mut stairs: Query<(&mut Transform, &mut Stair)>,
) {
    let ss = stair_param.span_sec;
    let delta = time.delta_secs();
    // let init_pos = stair.init_pos;

    for (mut transform, mut stair) in stairs.iter_mut() {
        transform.translation.y -= delta / ss;
        transform.translation.z += delta / ss;

        let y = transform.translation.y;

        // if y < - height * count then, loop
        if y < - stair_param.height * 3.0 {
            transform.translation = stair_param.last_stair_pos.unwrap();
        }
    }
}

fn swing_camera(
    time: Res<Time>,
    mut stair_param: ResMut<StairParam>,
    mut camera: Query<(&mut Transform, &mut Camera3d)>,
) {
    let n: f32 = 20.0;
    let t = (time.elapsed_secs_f64() % (n as f64)) as f32;
    let r = t / n * 2.0 * PI;
    for (mut transform, _) in camera.iter_mut() {
        transform.translation.x = sin_cos(r).0 * 3.0 - 5.0;
    }
}


#[cfg(feature = "egui")]
fn ui_system(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}