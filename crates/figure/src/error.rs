//! シーンの読み込みと検査で起こる誤り．

use std::fmt;

use semver::Version;

/// 誤りの種類．
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// JSONとして読めない．行と列は，1から数える．
    Json {
        /// 行．
        line: usize,
        /// 列．
        column: usize,
        /// 元の説明．
        message: String,
    },
    /// `version`がない．
    MissingVersion,
    /// `version`が，semverの版として読めない．中身は，書かれていた値．
    InvalidVersion(String),
    /// エンジンが読めない版で書かれている．
    IncompatibleVersion {
        /// シーンの版．
        scene: Version,
        /// エンジンの版．
        engine: Version,
    },
    /// 項目の誤り．未知の項目や種類，欠けた項目，型の違い．中身は，元の説明．
    Invalid(String),
    /// `id`が，英数字と`_`で書かれた識別子ではない．
    InvalidId(String),
    /// `id`が，他のオブジェクトと重なっている．
    DuplicateId(String),
    /// 変数の名前が，識別子ではない．
    InvalidVariable(String),
    /// 文字列の項目が空である．中身は，項目の名前．
    EmptyText(&'static str),
    /// 範囲が，有限の数で，下端が上端より小さくなっていない．中身は，項目の名前．
    InvalidRange(&'static str),
    /// 曲線の式の数が合わない．
    ExpressionCount {
        /// 必要な数．
        expected: usize,
        /// 書かれている数．
        found: usize,
    },
}

impl ErrorKind {
    /// 種類の名前．GUIやページが，誤りの種類で分岐するために使う．重ならない．
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Json { .. } => "json",
            Self::MissingVersion => "missing_version",
            Self::InvalidVersion(_) => "invalid_version",
            Self::IncompatibleVersion { .. } => "incompatible_version",
            Self::Invalid(_) => "invalid",
            Self::InvalidId(_) => "invalid_id",
            Self::DuplicateId(_) => "duplicate_id",
            Self::InvalidVariable(_) => "invalid_variable",
            Self::EmptyText(_) => "empty_text",
            Self::InvalidRange(_) => "invalid_range",
            Self::ExpressionCount { .. } => "expression_count",
        }
    }
}

/// 誤り．
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// 誤りの種類．
    pub kind: ErrorKind,
    /// 誤りのあるオブジェクトの`id`．GUIが，原因を指せるようにする．
    pub object: Option<String>,
}

impl Error {
    /// 誤りを作る．
    #[must_use]
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, object: None }
    }

    /// オブジェクトの`id`つきの誤りを作る．
    #[must_use]
    pub fn in_object(object: &str, kind: ErrorKind) -> Self {
        Self {
            kind,
            object: Some(object.to_owned()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(object) = &self.object {
            write!(f, "オブジェクト「{object}」：")?;
        }
        match &self.kind {
            ErrorKind::Json {
                line,
                column,
                message,
            } => write!(f, "JSONとして読めない({line}行{column}列)：{message}"),
            ErrorKind::MissingVersion => write!(f, "`version`がない．"),
            ErrorKind::InvalidVersion(value) => {
                write!(f, "`version`「{value}」は，1.2.3の形の版で書く．")
            }
            ErrorKind::IncompatibleVersion { scene, engine } => write!(
                f,
                "このシーンは版{scene}で書かれていて，版{engine}のエンジンでは読めない．"
            ),
            ErrorKind::Invalid(message) => write!(f, "{message}"),
            ErrorKind::InvalidId(id) => write!(
                f,
                "`id`「{id}」は，英字か数字で始め，英数字と`_`だけで書く．"
            ),
            ErrorKind::DuplicateId(id) => write!(f, "`id`「{id}」が重なっている．"),
            ErrorKind::InvalidVariable(name) => write!(
                f,
                "変数の名前「{name}」は，英字で始め，英数字と`_`だけで書く．"
            ),
            ErrorKind::EmptyText(field) => write!(f, "`{field}`が空である．"),
            ErrorKind::InvalidRange(field) => write!(
                f,
                "`{field}`は，有限の数で，下端が上端より小さい範囲にする．"
            ),
            ErrorKind::ExpressionCount { expected, found } => {
                write!(f, "式は{expected}個必要だが，{found}個書かれている．")
            }
        }
    }
}

impl std::error::Error for Error {}
