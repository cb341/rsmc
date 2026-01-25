// std crates
pub use std::collections::HashMap;
pub use std::f32::consts::*;
pub use std::{net::*, time::*};

// bevy crates
pub use bevy::asset::Assets;
pub use bevy::diagnostic::*;
pub use bevy::gizmos::gizmos::*;
pub use bevy::input::{keyboard::*, mouse::*, ButtonInput};
pub use bevy::math::{primitives::Cuboid, EulerRot, Quat, Ray3d, Vec3};
pub use bevy::pbr::*;
pub use bevy::prelude::*;
pub use bevy::transform::components::Transform;
pub use bevy::window::*;

pub use bevy_fps_controller::controller::*;

pub use bevy_rapier3d::geometry::Collider;
pub use bevy_rapier3d::{dynamics::*, geometry::*};
pub use bevy_rapier3d::{plugin::*, render::RapierDebugRenderPlugin};

// networking crates
pub use renet::{ClientId, ConnectionConfig, DefaultChannel, RenetClient};

// other crates
pub use rayon::iter::IntoParallelIterator;
pub use rayon::iter::IntoParallelRefMutIterator;
pub use rayon::iter::ParallelIterator;
pub use serde::*;

pub use self::terrain_util::Block;
pub use bevy_asset::prelude;
pub use noise::NoiseFn;
pub use noise::Perlin;
pub use terrain_util::CubeFace;

// my crates
pub use crate::states::GameState;
pub use lib::*;
pub use rsmc as lib;

pub use crate::collider::components as collider_components;
pub use crate::collider::events as collider_events;
pub use crate::collider::systems as collider_systems;

pub use crate::networking::commands as networking_commands;
pub use crate::networking::systems as networking_systems;
pub use crate::networking::NetworkingPlugin;

pub use crate::player::components as player_components;
pub use crate::player::events as player_events;
pub use crate::player::resources as player_resources;
pub use crate::player::systems as player_systems;

pub use crate::remote_player::components as remote_player_components;
pub use crate::remote_player::events as remote_player_events;
pub use crate::remote_player::systems as remote_player_systems;

pub use crate::terrain::components as terrain_components;
pub use crate::terrain::events as terrain_events;
pub use crate::terrain::resources as terrain_resources;
pub use crate::terrain::systems as terrain_systems;
pub use crate::terrain::util as terrain_util;

pub use crate::gui::components as gui_components;
pub use crate::gui::events as gui_events;
pub use crate::gui::systems as gui_systems;

pub use crate::chat::components as chat_components;
pub use crate::chat::events as chat_events;
pub use crate::chat::resources as chat_resources;
pub use crate::chat::systems as chat_systems;
