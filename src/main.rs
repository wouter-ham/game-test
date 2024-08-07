use bevy::input::ButtonState;
use bevy::input::gestures::{PinchGesture, RotationGesture};
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::prelude::*;

use crate::camera::{CameraAction, PanOrbitCameraBundle, PanOrbitSettings, PanOrbitState};

mod camera;

#[derive(Component)]
struct Ground;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .add_systems(Update, pan_orbit_camera)
        .run();
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) = ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    gizmos.circle(point + ground.up() * 0.01, ground.up(), 0.2, Color::WHITE);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(20., 20.)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.5)),
            ..default()
        },
        Ground,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PanOrbitCameraBundle::default());
}

fn pan_orbit_camera(
    mut evr_mouse: EventReader<MouseButtonInput>,
    mut evr_gesture_pinch: EventReader<PinchGesture>,
    mut evr_gesture_rotate: EventReader<RotationGesture>,
    mut evr_motion: EventReader<MouseMotion>,
    mut q_camera: Query<(
        &PanOrbitSettings,
        &mut PanOrbitState,
        &mut Transform,
    )>,
) {
    for (settings, mut state, mut transform) in &mut q_camera {
        let mut total_motion: Vec2 = evr_motion.read().map(|ev: &MouseMotion| ev.delta).sum();

        // Reverse Y (Bevy's Worldspace coordinate system is Y-Up,
        // but events are in window/ui coordinates, which are Y-Down)
        total_motion.y = -total_motion.y;

        let mut camera_action: CameraAction = state.action;

        if total_motion != Vec2::ZERO {
            camera_action = CameraAction::Move;
        }

        let mut total_zoom_pixels: f32 = evr_gesture_pinch.read().map(|ev: &PinchGesture| ev.0).sum();
        if total_zoom_pixels != 0. {
            println!("zooming: {}", total_zoom_pixels);
            camera_action = CameraAction::Zoom;
        }

        let mut total_rotation: f32 = evr_gesture_rotate.read().map(|ev: &RotationGesture| ev.0).sum();
        if total_rotation != 0. {
            println!("rotating: {}", total_rotation);
            camera_action = CameraAction::Rotate;
        }

        for ev in evr_mouse.read() {
            state.mouse_state = ev.state;
            println!("mouse state: {:?}", state.mouse_state);
        }

        match (camera_action, state.mouse_state) {
            (CameraAction::Move, ButtonState::Pressed) => {
                camera_action = CameraAction::Move;
            }
            (CameraAction::Move, ButtonState::Released) => {
                camera_action = CameraAction::None;
            }
            _ => {}
        }

        state.action = camera_action;

        match state.action {
            CameraAction::Move => {
                let mut total_move: Vec2 = Vec2::ZERO;
                total_move = total_motion * settings.move_sensitivity;

                println!("total_move: {}", total_move);

                // let radius = state.radius;
                state.center -= transform.right() * total_move.x;
                state.center -= transform.forward() * total_move.y;
            }
            CameraAction::Zoom => {
                let total_zoom: f32 = total_zoom_pixels * settings.zoom_sensitivity;

                println!("total_zoom: {}", total_zoom);

                state.radius *= (-total_zoom).exp();
            }
            // CameraAction::Rotate => {
            //     total_rotation *= settings.rotate_sensitivity;
            //
            //     println!("total_rotation: {}", total_rotation);
            //
            //     state.rotation *= (-total_rotation).exp();
            // }
            _ => {}
        }

        if camera_action != CameraAction::None || state.is_added() {
            transform.rotation = Quat::from_euler(EulerRot::YXZ, state.rotation, state.slope, 0.0);
            // To position the camera, get the backward direction vector
            // and place the camera at the desired radius from the center.
            transform.translation = state.center + transform.back() * state.radius;
        }
    }
}
