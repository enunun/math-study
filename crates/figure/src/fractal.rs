//! フラクタル図形．反復関数系(IFS)：基本図形を，アフィン変換の集まりで再帰的に写す．
//! アフィン変換は，ほかのオブジェクトの変換(`transform.rs`)と同じものを使う．

use crate::transform::Affine;

/// 基本図形(`base`)を，`transforms`で`depth`回，再帰的に写す．深さ0は`base`そのもの，深さ`n`は，
/// 深さ`n - 1`の図形全体に，それぞれの変換を施したものをすべて集めたものである．
/// `all_depths`なら，深さ0から`depth`までの図形をすべて返す(浅い順)．
#[must_use]
pub fn instances(
    base: &[[f64; 2]],
    transforms: &[Affine],
    depth: u32,
    all_depths: bool,
) -> Vec<Vec<[f64; 2]>> {
    let mut level = vec![base.to_vec()];
    let mut collected = Vec::new();
    for _ in 0..depth {
        let mut next = Vec::with_capacity(level.len().saturating_mul(transforms.len()));
        for transform in transforms {
            for path in &level {
                next.push(path.iter().map(|point| transform.apply2(*point)).collect());
            }
        }
        if all_depths {
            collected.append(&mut level);
        }
        level = next;
    }
    collected.append(&mut level);
    collected
}
