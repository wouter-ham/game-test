use bevy::input::ButtonState;
use bevy::math::Vec3;
use bevy::prelude::{Bundle, Camera3dBundle, Component};

// Bundle to spawn our custom camera easily
#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub camera: Camera3dBundle,
    pub state: PanOrbitState,
    pub settings: PanOrbitSettings,
}

// The internal state of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitState {
    pub center: Vec3,
    pub radius: f32,
    pub slope: f32,
    pub rotation: f32,
    pub action: CameraAction,
    pub mouse_state: ButtonState,
}

/// The configuration of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitSettings {
    /// Exponent per pixel of mouse motion
    pub zoom_sensitivity: f32,
    /// Move sensitivity
    pub move_sensitivity: f32,
    /// Rotation sensitivity
    pub rotate_sensitivity: f32,
    /// For devices with a notched scroll wheel, like desktop mice
    pub scroll_line_sensitivity: f32,
    /// For devices with smooth scrolling, like touchpads
    pub scroll_pixel_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraAction {
    Rotate,
    Zoom,
    Move,
    None,
}

impl Default for PanOrbitState {
    fn default() -> Self {
        PanOrbitState {
            center: Vec3::new(0.0, 2.0, 15.0), // width, height, depth
            radius: 1.0,
            slope: 0.0,
            rotation: 0.0,
            action: CameraAction::None,
            mouse_state: ButtonState::Released,
        }
    }
}

impl Default for PanOrbitSettings {
    fn default() -> Self {
        PanOrbitSettings {
            zoom_sensitivity: 5.0,
            move_sensitivity: 0.1,
            rotate_sensitivity: 0.1,
            scroll_line_sensitivity: 16.0, // 1 "line" == 16 "pixels of motion"
            scroll_pixel_sensitivity: 1.0,
        }
    }
}
