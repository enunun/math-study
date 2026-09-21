//! 図のシーン(JSON)を読み，検査する．
//!
//! シーンは，図の入力だけを持つデータである(点，媒介変数，式，スタイル)．
//! 入口は，`parse_scene`である．誤りは，原因のオブジェクトの`id`を持つ．

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::panic
    )
)]

pub mod arrow;
pub mod clip;
mod compile;
pub mod error;
pub mod expr;
pub mod figure;
mod parse;
mod render;
pub mod sample;
pub mod scene;
mod space;
pub mod tikz;
mod validate;
pub mod version;

pub use error::{Error, ErrorKind};
pub use parse::parse_scene;
pub use render::render;
pub use scene::Scene;
