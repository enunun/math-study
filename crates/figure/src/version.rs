//! シーンの版と，エンジンの版の互換性．

use semver::Version;

/// エンジンの版．Rustのワークスペースの版と同じである．
///
/// Cargoが保証する版なので，読めないことはない．テストで，`CARGO_PKG_VERSION`と同じことを確かめる．
#[must_use]
pub fn engine_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or_else(|_| Version::new(0, 0, 0))
}

/// エンジンが，その版で書かれたシーンを読めるか．
///
/// 主版が同じで(主版が0の間は，副版も同じ)，エンジンより新しくない版なら読める．
#[must_use]
pub fn is_readable(scene: &Version, engine: &Version) -> bool {
    scene <= engine
        && scene.major == engine.major
        && (engine.major != 0 || scene.minor == engine.minor)
}
