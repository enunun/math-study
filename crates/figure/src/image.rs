//! 像(`image`)を，元のオブジェクトの複製に置き換える．複製は，像の`id`を持ち，元の変換のあとに
//! 像の変換を施す．置き換えたシーンは，ほかのオブジェクトと同じに検査し，描く．

use std::borrow::Cow;

use crate::error::{Error, ErrorKind};
use crate::scene::{Image, Object, Scene, Style, TransformStep};

/// 像を複製に置き換えたシーン．像がなければ，元のシーンのままである．
///
/// # Errors
///
/// 像の`of`が，先に置いたオブジェクトでないか，変換できない種類なら，誤りを返す．
pub fn expand_images(scene: &Scene) -> Result<Cow<'_, Scene>, Error> {
    if !scene
        .objects
        .iter()
        .any(|object| matches!(object, Object::Image(_)))
    {
        return Ok(Cow::Borrowed(scene));
    }
    let mut objects: Vec<Object> = Vec::with_capacity(scene.objects.len());
    for object in &scene.objects {
        let expanded = match object {
            Object::Image(image) => {
                let original = objects
                    .iter()
                    .find(|placed| placed.id() == image.of)
                    .ok_or_else(|| {
                        Error::in_object(
                            &image.id,
                            ErrorKind::Invalid(format!(
                                "像の元「{}」がない．`of`には，像より前に置いたオブジェクトの`id`を書く．",
                                image.of
                            )),
                        )
                    })?;
                copy_of(original, image).map_err(|kind| Error::in_object(&image.id, kind))?
            }
            other => other.clone(),
        };
        objects.push(expanded);
    }
    Ok(Cow::Owned(Scene {
        objects,
        ..scene.clone()
    }))
}

/// 元のオブジェクトの複製に，像の`id`，変換，スタイルを与える．
fn copy_of(original: &Object, image: &Image) -> Result<Object, ErrorKind> {
    let mut copy = original.clone();
    if image.label.is_some() && !matches!(copy, Object::Point(_)) {
        return Err(ErrorKind::Invalid(
            "像の名前(`label`)は，点の像にだけ書ける．".to_owned(),
        ));
    }
    let Some((id, transform, style)) = parts_of(&mut copy) else {
        return Err(ErrorKind::Invalid(format!(
            "「{}」(`{}`)は変換できない．座標軸・媒介変数・関数・写像は，像の元にできない．",
            original.id(),
            original.type_name()
        )));
    };
    image.id.clone_into(id);
    transform.extend(image.transform.iter().cloned());
    if !image.style.is_default() {
        let Some(style) = style else {
            return Err(ErrorKind::Invalid(
                "ラベルの像には，スタイル(`style`)を書けない．".to_owned(),
            ));
        };
        *style = image.style;
    }
    if let Object::Point(point) = &mut copy {
        point.label.clone_from(&image.label);
    }
    Ok(copy)
}

/// 変換できるオブジェクトの，`id`，変換，スタイル(ラベルは持たない)．変換できない種類なら`None`．
type Parts<'a> = (
    &'a mut String,
    &'a mut Vec<TransformStep>,
    Option<&'a mut Style>,
);

fn parts_of(object: &mut Object) -> Option<Parts<'_>> {
    Some(match object {
        Object::Label(o) => (&mut o.id, &mut o.transform, None),
        Object::Point(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Segment(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Vector(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Graph(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Curve(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Polygon(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Fractal(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Grid(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Surface(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Complex(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Polyhedron(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::TangentLine(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Region(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Taylor(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Sphere(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Cut(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::Intersection(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        Object::TangentPlane(o) => (&mut o.id, &mut o.transform, Some(&mut o.style)),
        _ => return None,
    })
}

/// 変換(`transform`)を持つオブジェクトの，変換の手順．持たない種類(座標軸・媒介変数・関数・写像・像)なら`None`．
/// 像の変換は，`expand_images`が複製に移す．
#[must_use]
pub fn transform_of(object: &Object) -> Option<&[TransformStep]> {
    Some(match object {
        Object::Point(o) => &o.transform,
        Object::Segment(o) => &o.transform,
        Object::Vector(o) => &o.transform,
        Object::Graph(o) => &o.transform,
        Object::Curve(o) => &o.transform,
        Object::Polygon(o) => &o.transform,
        Object::Fractal(o) => &o.transform,
        Object::Grid(o) => &o.transform,
        Object::Surface(o) => &o.transform,
        Object::Complex(o) => &o.transform,
        Object::Polyhedron(o) => &o.transform,
        Object::Label(o) => &o.transform,
        Object::TangentLine(o) => &o.transform,
        Object::Region(o) => &o.transform,
        Object::Taylor(o) => &o.transform,
        Object::Sphere(o) => &o.transform,
        Object::Cut(o) => &o.transform,
        Object::Intersection(o) => &o.transform,
        Object::TangentPlane(o) => &o.transform,
        _ => return None,
    })
}
