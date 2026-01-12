pub mod blocks;
pub mod buffer_serializer;
pub mod chunk;
pub mod chunk_serializer;
pub mod networking;

pub use blocks::*;
pub use buffer_serializer::*;
pub use chunk::*;
pub use networking::*;

#[macro_export]
macro_rules! single_mut {
    ($q:expr) => {
        match $q.single_mut() {
            Ok(m) => m,
            _ => return,
        }
    };
}

#[macro_export]
macro_rules! single {
    ($q:expr) => {
        match $q.single() {
            Ok(m) => m,
            _ => return,
        }
    };
}
