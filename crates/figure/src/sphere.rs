//! 変換(`transform`)を持つ球を，同じ形の曲面(`surface`)に置き換える．球を変換すると，楕円面のように
//! 球でなくなることがあるので，球の輪郭の円や，球による隠れ方の計算は使えない．曲面なら，変換を式に
//! 施して，輪郭も隠れ方もワイヤーフレームも，ほかの曲面と同じに求まる．

use std::borrow::Cow;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::scene::{Bound, Object, Scene, Sphere, Surface};
use crate::space::{SPHERE_MERIDIANS, SPHERE_PARALLELS};

/// 変換を持つ球を，曲面に置き換えたシーン．そのような球がなければ，元のシーンのままである．
#[must_use]
pub fn expand_transformed_spheres(scene: Cow<'_, Scene>) -> Cow<'_, Scene> {
    let transformed =
        |object: &Object| matches!(object, Object::Sphere(sphere) if !sphere.transform.is_empty());
    if !scene.objects.iter().any(transformed) {
        return scene;
    }
    let mut owned = scene.into_owned();
    for object in &mut owned.objects {
        if let Object::Sphere(sphere) = object
            && !sphere.transform.is_empty()
        {
            *object = Object::Surface(as_surface(sphere));
        }
    }
    Cow::Owned(owned)
}

/// 球と同じ形の曲面．変数は経度`sphere_theta`と緯度`sphere_phi`で，ワイヤーフレームの刻みは，球の経線と
/// 緯線と同じ所に断面が来るように決める．経度の範囲を刻みの半分だけずらし，経度0の経線も内側に入れる．
fn as_surface(sphere: &Sphere) -> Surface {
    let [cx, cy, cz] = sphere.center;
    let r = sphere.radius;
    let meridian_step = TAU / f64::from(u32::try_from(SPHERE_MERIDIANS).unwrap_or(1));
    let parallel_step =
        PI / f64::from(u32::try_from(SPHERE_PARALLELS.saturating_add(1)).unwrap_or(1));
    Surface {
        id: sphere.id.clone(),
        vars: vec!["sphere_theta".to_owned(), "sphere_phi".to_owned()],
        expr: vec![
            format!("({cx}) + ({r})*cos(sphere_phi)*cos(sphere_theta)"),
            format!("({cy}) + ({r})*cos(sphere_phi)*sin(sphere_theta)"),
            format!("({cz}) + ({r})*sin(sphere_phi)"),
        ],
        domain: Some([
            [
                Bound::Number(-meridian_step / 2.0),
                Bound::Number(TAU - meridian_step / 2.0),
            ],
            [Bound::Number(-FRAC_PI_2), Bound::Number(FRAC_PI_2)],
        ]),
        bezier: None,
        mesh: [Surface::DEFAULT_MESH; 2],
        boundary: false,
        style: sphere.style,
        wireframe: sphere.wireframe,
        wireframe_step: Some([Bound::Number(meridian_step), Bound::Number(parallel_step)]),
        control_net: None,
        transform: sphere.transform.clone(),
    }
}
