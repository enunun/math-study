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

pub mod error;
mod parse;
pub mod scene;
mod validate;
pub mod version;

pub use error::{Error, ErrorKind};
pub use parse::parse_scene;
pub use scene::Scene;
