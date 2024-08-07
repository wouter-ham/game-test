use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::camera::{PanOrbitAction, PanOrbitCameraBundle, PanOrbitSettings, PanOrbitState};

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
    let Some(distance) =
        ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    else {
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
    // mbi: Res<MouseButtonInput>,
    mut evr_gesture_pinch: EventReader<PinchGesture>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &PanOrbitSettings,
        &mut PanOrbitState,
        &mut Transform,
    )>,
) {
    // First, accumulate the total amount of
    // mouse motion and scroll, from all pending events:
    let mut total_motion: Vec2 = evr_motion.read()
        .map(|ev| ev.delta).sum();

    // Reverse Y (Bevy's Worldspace coordinate system is Y-Up,
    // but events are in window/ui coordinates, which are Y-Down)
    total_motion.y = -total_motion.y;

    let mut total_scroll_lines = Vec2::ZERO;
    let mut total_scroll_pixels = Vec2::ZERO;
    for ev in evr_scroll.read() {
        println!("scrolling");
        match ev.unit {
            MouseScrollUnit::Line => {
                total_scroll_lines.x += ev.x;
                total_scroll_lines.y -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_scroll_pixels.x += ev.x;
                total_scroll_pixels.y -= ev.y;
            }
        }
    }

    let mut total_zoom_pixels = 0.;
    for ev in evr_gesture_pinch.read() {
        total_zoom_pixels += ev.0;
        println!("zooming: {}, {}", total_zoom_pixels, total_motion);
    }

    let mut pan_orbit_action: PanOrbitAction = PanOrbitAction::Pan;

    // if total_scroll_lines != Vec2::ZERO || total_scroll_pixels != Vec2::ZERO {
    //     pan_orbit_action = PanOrbitAction::Orbit;
    // }

    if total_zoom_pixels != 0. {
        pan_orbit_action = PanOrbitAction::Zoom;
    }

    for (settings, mut state, mut transform) in &mut q_camera {
        // Check how much of each thing we need to apply.
        // Accumulate values from motion and scroll,
        // based on our configuration settings.

        let mut total_pan: Vec2 = Vec2::ZERO;
        let mut total_orbit: Vec2 = Vec2::ZERO;
        let mut total_zoom: Vec2 = Vec2::ZERO;

        match pan_orbit_action {
            PanOrbitAction::Pan => {
                total_pan -= total_scroll_lines
                    * settings.scroll_line_sensitivity * settings.pan_sensitivity;
                total_pan -= total_scroll_pixels
                    * settings.scroll_pixel_sensitivity * settings.pan_sensitivity;
            }
            PanOrbitAction::Orbit => {
                total_orbit -= total_scroll_lines
                    * settings.scroll_line_sensitivity * settings.orbit_sensitivity;
                total_orbit -= total_scroll_pixels
                    * settings.scroll_pixel_sensitivity * settings.orbit_sensitivity;
            }
            PanOrbitAction::Zoom => {
                total_zoom += total_zoom_pixels * settings.zoom_sensitivity;
            }
        }

        // Pan
        // if settings.pan_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
        //     total_pan -= total_motion * settings.pan_sensitivity;
        // }
        // if settings.scroll_action == Some(PanOrbitAction::Pan) {
        //
        // }

        // Orbit
        // if settings.orbit_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
        //     total_orbit -= total_motion * settings.orbit_sensitivity;
        // }


        // Upon starting a new orbit maneuver (key is just pressed),
        // check if we are starting it upside-down
        // if settings.orbit_key.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
        state.upside_down = state.slope < -FRAC_PI_2 || state.slope > FRAC_PI_2;
        // }

        // If we are upside down, reverse the X orbiting
        if state.upside_down {
            total_orbit.x = -total_orbit.x;
        }

        // Now we can actually do the things!

        let mut any = false;

        // To ZOOM, we need to multiply our radius.
        if total_zoom != Vec2::ZERO {
            any = true;
            // in order for zoom to feel intuitive,
            // everything needs to be exponential
            // (done via multiplication)
            // not linear
            // (done via addition)

            // so we compute the exponential of our
            // accumulated value and multiply by that
            state.radius *= (-total_zoom.y).exp();
        }

        // To ORBIT, we change our slope and rotation values
        if total_orbit != Vec2::ZERO {
            any = true;
            state.rotation += total_orbit.x;
            state.slope += total_orbit.y;
            // wrap around, to stay between +- 180 degrees
            if state.rotation > PI {
                state.rotation -= TAU; // 2 * PI
            }
            if state.rotation < -PI {
                state.rotation += TAU; // 2 * PI
            }
            if state.slope > PI {
                state.slope -= TAU; // 2 * PI
            }
            if state.slope < -PI {
                state.slope += TAU; // 2 * PI
            }
        }

        // To PAN, we can get the UP and RIGHT direction
        // vectors from the camera's transform, and use
        // them to move the center point. Multiply by the
        // radius to make the pan adapt to the current zoom.
        if total_pan != Vec2::ZERO {
            any = true;
            let radius = state.radius;
            state.center += transform.right() * total_pan.x * radius;
            state.center += transform.up() * total_pan.y * radius;
        }

        // Finally, compute the new camera transform.
        // (if we changed anything, or if the pan-orbit
        // controller was just added and thus we are running
        // for the first time and need to initialize)
        if any || state.is_added() {
            // YXZ Euler Rotation performs rotation/slope/roll.
            transform.rotation =
                Quat::from_euler(EulerRot::YXZ, state.rotation, state.slope, 0.0);
            // To position the camera, get the backward direction vector
            // and place the camera at the desired radius from the center.
            transform.translation = state.center + transform.back() * state.radius;
        }
    }
}
