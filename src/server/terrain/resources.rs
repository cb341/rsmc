use std::collections::VecDeque;

use crate::prelude::*;

use chrono::{DateTime, TimeDelta, Utc};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use terrain_events::BlockUpdateEvent;

#[derive(Resource, Default)]
pub struct ClientChunkRequests {
    queues: HashMap<ClientId, VecDeque<IVec3>>,
}

impl ClientChunkRequests {
    pub fn enqueue_bulk(&mut self, client_id: ClientId, chunk_positions: &mut VecDeque<IVec3>) {
        self.queues
            .entry(client_id)
            .or_default()
            .append(chunk_positions);
    }

    pub fn remove(&mut self, client_id: &ClientId) {
        self.queues.remove(client_id);
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&ClientId, &mut VecDeque<IVec3>) -> bool,
    {
        self.queues.retain(f)
    }
}

#[derive(Resource)]
pub struct AutoSaveName(pub String);

impl AutoSaveName {
    pub fn with_name(name: String) -> Self {
        Self(name)
    }

    pub fn with_random() -> Self {
        Self(Alphanumeric.sample_string(&mut rand::rng(), 16))
    }
}

#[derive(Resource)]
struct SaveTimer {
    pub last_autosave_timestamp: DateTime<Utc>,
    interval: chrono::TimeDelta,
}

impl SaveTimer {
    pub fn new(interval: TimeDelta) -> SaveTimer {
        SaveTimer {
            last_autosave_timestamp: Utc::now(),
            interval,
        }
    }
}

#[derive(Resource)]
pub struct WorldBackupTimer(SaveTimer);

impl WorldBackupTimer {
    pub fn reset(&mut self) {
        self.0.reset()
    }

    pub fn is_ready(&self) -> bool {
        self.0.is_ready()
    }
}

impl Default for WorldBackupTimer {
    fn default() -> Self {
        Self(SaveTimer::new(TimeDelta::seconds(180)))
    }
}

#[derive(Resource)]
pub struct WorldSaveTimer(SaveTimer);

impl WorldSaveTimer {
    pub fn reset(&mut self) {
        self.0.reset()
    }

    pub fn is_ready(&self) -> bool {
        self.0.is_ready()
    }
}

impl Default for WorldSaveTimer {
    fn default() -> Self {
        Self(SaveTimer::new(TimeDelta::seconds(30)))
    }
}

impl SaveTimer {
    pub fn reset(&mut self) {
        self.last_autosave_timestamp = Utc::now();
    }

    pub fn is_ready(&self) -> bool {
        let timer_ready_timestamp = self
            .last_autosave_timestamp
            .checked_add_signed(self.interval)
            .expect("Time should never be out of range");
        timer_ready_timestamp < Utc::now()
    }
}

#[derive(Resource, Default)]
pub struct PastBlockUpdates {
    pub updates: Vec<BlockUpdateEvent>,
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Generator {
    pub seed: u32,
    pub params: TerrainGeneratorParams,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub perlin: Perlin,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HeightParams {
    pub noise: NoiseFunctionParams,
    pub splines: Vec<Vec2>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DensityParams {
    pub noise: NoiseFunctionParams,
    pub squash_factor: f64,
    pub height_offset: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CaveParams {
    pub noise: NoiseFunctionParams,
    pub base_value: f64,
    pub threshold: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HeightAdjustParams {
    pub noise: NoiseFunctionParams,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GrassParams {
    pub frequency: u32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct NoiseFunctionParams {
    pub octaves: u32,
    pub height: f64,
    pub lacuranity: f64,
    pub frequency: f64,
    pub amplitude: f64,
    pub persistence: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TreeParams {
    pub spawn_attempts_per_chunk: u32,
    pub min_stump_height: u32,
    pub max_stump_height: u32,
    pub min_bush_radius: u32,
    pub max_bush_radius: u32,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Generator {
    pub fn with_seed(seed: u32) -> Self {
        Self::new(seed)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainGeneratorParams {
    pub height: HeightParams,
    pub height_adjust: HeightAdjustParams,
    pub density: DensityParams,
    pub cave: CaveParams,
    pub tree: TreeParams,
    pub grass: GrassParams,
}

impl Default for TerrainGeneratorParams {
    fn default() -> Self {
        Self {
            height: HeightParams {
                splines: vec![
                    Vec2::new(-1.0, 4.0),
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.0, 0.0),
                    Vec2::new(0.05, 20.0),
                    Vec2::new(1.0, 35.0),
                ],
                noise: NoiseFunctionParams {
                    octaves: 4,
                    height: 0.0,
                    lacuranity: 2.0,
                    frequency: 1.0 / 300.0,
                    amplitude: 30.0,
                    persistence: 0.5,
                },
            },
            height_adjust: HeightAdjustParams {
                noise: NoiseFunctionParams {
                    octaves: 4,
                    height: 0.0,
                    lacuranity: 2.0,
                    frequency: 1.0 / 120.0,
                    amplitude: 30.0,
                    persistence: 0.5,
                },
            },
            density: DensityParams {
                squash_factor: 1.0 / 100.0,
                height_offset: -20.0,
                noise: NoiseFunctionParams {
                    octaves: 4,
                    height: 0.0,
                    lacuranity: 2.0,
                    frequency: 1.0 / 60.0,
                    amplitude: 10.0,
                    persistence: 0.5,
                },
            },
            cave: CaveParams {
                noise: NoiseFunctionParams {
                    octaves: 2,
                    height: 0.0,
                    lacuranity: 0.03,
                    frequency: 1.0 / 20.0,
                    amplitude: 30.0,
                    persistence: 0.59,
                },
                base_value: 0.0,
                threshold: 0.25,
            },
            tree: TreeParams {
                spawn_attempts_per_chunk: 500,
                min_stump_height: 2,
                max_stump_height: 20,
                min_bush_radius: 3,
                max_bush_radius: 5,
            },
            grass: GrassParams { frequency: 10 },
        }
    }
}

#[cfg(feature = "generator_visualizer")]
pub use visualizer::*;

#[cfg(feature = "generator_visualizer")]
mod visualizer {
    use super::*;
    use bevy_inspector_egui::egui::TextureHandle;

    #[derive(PartialEq, Hash, Eq, Clone, Debug)]
    pub enum TextureType {
        Height,
        HeightAdjust,
        Density,
        Cave,
    }

    #[derive(Resource)]
    pub struct NoiseTextureList {
        pub noise_textures: HashMap<TextureType, NoiseTexture>,
    }

    impl Default for NoiseTextureList {
        fn default() -> Self {
            let mut noise_textures = HashMap::new();

            noise_textures.insert(TextureType::Height, NoiseTexture::default());
            noise_textures.insert(TextureType::HeightAdjust, NoiseTexture::default());
            noise_textures.insert(TextureType::Density, NoiseTexture::default());
            noise_textures.insert(TextureType::Cave, NoiseTexture::default());

            NoiseTextureList { noise_textures }
        }
    }

    pub struct NoiseTexture {
        pub texture: Option<TextureHandle>,
        pub size: Vec2,
    }

    impl Default for NoiseTexture {
        fn default() -> Self {
            NoiseTexture {
                texture: None,
                size: Vec2::ZERO,
            }
        }
    }
}
