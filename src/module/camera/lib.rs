//! # Look Transform
//!
//! All controllers are based on a [`LookTransform`] component, which is just an
//! `eye` point that looks at a `target` point. By modifying this component, the
//! scene graph `Transform` will automatically be synchronized.
//!
//! Any entities with all of `Transform`, LookTransform, and [`Smoother`]
//! components will automatically have their `Transform` smoothed. Smoothing
//! will have no effect on the `LookTransform`, only the final `Transform` in
//! the scene graph.
//!
//! ```rust
//! use bevy::prelude::*;
//! use crate::module::camera::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         // Enables the system that synchronizes your `Transform`s and `LookTransform`s.
//!         .add_plugins(LookTransformPlugin)
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, move_camera_system);
//! }
//!
//! fn setup(mut commands: Commands) {
//!     let eye = Vec3::default();
//!     let target = Vec3::default();
//!
//!     commands
//!         .spawn((Camera3d::default(),
//                  LookTransform::new(eye, target, Vec3::Y),
//!                 Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
//!         ));
//!
//! }
//!
//! fn move_camera_system(mut cameras: Query<&mut LookTransform>) {
//!     // Later, another system will update the `Transform` and apply smoothing automatically.
//!     for mut c in cameras.iter_mut() { c.target += Vec3::new(1.0, 1.0, 1.0); }
//! }
//! ```
//!
//! # Look Angles
//!
//! When implementing a camera controller, it's often useful to work directly
//! with the angles (pitch and yaw) of your look direction. You can do this with
//! the [`LookAngles`] type:
//!
//! ```rust
//! use bevy::prelude::*;
//! use crate::module::camera::{
//!     LookAngles,
//!     LookTransform
//! };
//!
//! fn look_angles(mut transform: LookTransform, delta: Vec2) {
//!     let mut angles = LookAngles::from_vector(transform.look_direction().unwrap());
//!     angles.add_pitch(delta.y);
//!     angles.add_yaw(delta.x);
//!     // Third-person.
//!     transform.eye = transform.target + 1.0 * transform.radius() * angles.unit_vector();
//!     // First-person.
//!     // transform.target = transform.eye + 1.0 * transform.radius() * angles.unit_vector();
//! }
//! ```
//!
//! This is how the built-in controllers implement rotation controls.
//!
//! # Built-In Controllers
//!
//! These plugins depend on the [`LookTransformPlugin`]:
//!
//! - [`FpsCameraPlugin`](crate::module::camera::controllers::fps::FpsCameraPlugin) +
//!   [`FpsCameraBundle`](crate::module::camera::controllers::fps::FpsCameraBundle)
//!   - WASD: Translate on the XZ plane
//!   - Shift/Space: Translate along the Y axis
//!   - Mouse: Rotate camera
//! - [`OrbitCameraPlugin`](crate::module::camera::controllers::orbit::OrbitCameraPlugin) +
//!   [`OrbitCameraBundle`](crate::module::camera::controllers::orbit::OrbitCameraBundle)
//!   - CTRL + mouse drag: Rotate camera
//!   - Right mouse drag: Pan camera
//!   - Mouse wheel: Zoom
//! - [`UnrealCameraPlugin`](crate::module::camera::controllers::unreal::UnrealCameraPlugin) +
//!   [`UnrealCameraBundle`](crate::module::camera::controllers::unreal::UnrealCameraBundle)
//!
//!   Best use: hold Right mouse button to orbit the view while using WASD to
//!   navigate in the scene, using scroll wheel to accelerate/decelerate.
//!   - Left mouse drag: Locomotion
//!   - Right mouse drag: Rotate camera
//!   - Left and Right or Middle mouse drag: Pan camera
//!   - While holding any mouse button, use A/D for panning left/right, Q/E for
//!     panning up/down
//!   - While holding any mouse button, use W/S for locomotion forward/backward
//!   - While holding any mouse button, use scroll wheel to increase/decrease
//!     locomotion and panning speeds
//!   - While holding no mouse button, use scroll wheel for locomotion
//!     forward/backward

use bevy::app::Plugin;

pub mod controllers;

mod look_angles;
mod look_transform;

pub use look_angles::*;
pub use look_transform::*;