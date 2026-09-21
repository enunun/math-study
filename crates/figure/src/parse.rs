//! シーンのJSONを読み込む．

use semver::Version;
use serde::Deserialize;
use serde_json::Value;

use crate::error::{Error, ErrorKind};
use crate::scene::{Object, Scene, View};
use crate::validate::validate;
use crate::version::{engine_version, is_readable};

/// オブジェクトを，型に読む前の値のまま持つ，シーンの外側の形．
///
/// 誤りに，原因のオブジェクトの`id`を付けるため，オブジェクトは1つずつ型に読む．
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScene {
    version: Version,
    description: String,
    view: View,
    objects: Vec<Value>,
}

/// シーンのJSONを読み，検査する．
///
/// # Errors
///
/// JSONの誤り，版の不一致，項目の誤り，`id`の誤りがあると，誤りを返す．
pub fn parse_scene(json: &str) -> Result<Scene, Error> {
    let value: Value = serde_json::from_str(json).map_err(|error| {
        Error::new(ErrorKind::Json {
            line: error.line(),
            column: error.column(),
            message: error.to_string(),
        })
    })?;
    check_version(&value)?;
    let raw: RawScene = serde_json::from_value(value).map_err(|error| invalid(&error))?;
    let objects = raw
        .objects
        .into_iter()
        .map(parse_object)
        .collect::<Result<Vec<_>, _>>()?;
    let scene = Scene {
        version: raw.version,
        description: raw.description,
        view: raw.view,
        objects,
    };
    validate(&scene)?;
    Ok(scene)
}

fn invalid(error: &serde_json::Error) -> Error {
    Error::new(ErrorKind::Invalid(error.to_string()))
}

fn parse_object(value: Value) -> Result<Object, Error> {
    let id = value.get("id").and_then(Value::as_str).map(str::to_owned);
    serde_json::from_value(value).map_err(|error| Error {
        kind: ErrorKind::Invalid(error.to_string()),
        object: id,
    })
}

/// `version`を，他の項目より先に検査する．新しい版のシーンは，未知の項目より先に，版の誤りにする．
fn check_version(value: &Value) -> Result<(), Error> {
    let Value::Object(fields) = value else {
        return Err(Error::new(ErrorKind::Invalid(
            "シーンは，JSONのオブジェクトで書く．".to_owned(),
        )));
    };
    let written = fields
        .get("version")
        .ok_or_else(|| Error::new(ErrorKind::MissingVersion))?;
    let text = written
        .as_str()
        .map_or_else(|| written.to_string(), str::to_owned);
    let scene: Version = text
        .parse()
        .map_err(|_| Error::new(ErrorKind::InvalidVersion(text.clone())))?;
    let engine = engine_version();
    if is_readable(&scene, &engine) {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::IncompatibleVersion { scene, engine }))
    }
}
