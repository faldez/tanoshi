use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::marker::PhantomData;
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    str::FromStr,
};

use crate::error::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split('.').collect::<Vec<&str>>();
        match split.as_slice() {
            [major, minor, patch] => Ok(Version {
                major: major.parse().map_err(|_| Error::InvalidVersion)?,
                minor: minor.parse().map_err(|_| Error::InvalidVersion)?,
                patch: patch.parse().map_err(|_| Error::InvalidVersion)?,
            }),
            _ => Err(Error::InvalidVersion),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.major, self.minor, self.patch))
    }
}

// https://serde.rs/string-or-struct.html
fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Error>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Error>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

/// A type represent source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub url: String,
    #[serde(deserialize_with = "string_or_struct")]
    pub version: Version,
    #[serde(default, deserialize_with = "string_or_struct")]
    pub lib_version: Version,
    pub icon: String,
    pub need_login: bool,
    #[serde(default = "Vec::new")]
    pub languages: Vec<String>,
}

impl Default for Source {
    fn default() -> Self {
        Source {
            id: 0,
            name: "".to_string(),
            url: "".to_string(),
            version: Version::default(),
            lib_version: Version::default(),
            icon: "".to_string(),
            need_login: false,
            languages: Vec::new(),
        }
    }
}

/// A type represent manga details, normalized across source
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manga {
    pub source_id: i64,
    pub title: String,
    pub author: Vec<String>,
    pub genre: Vec<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub cover_url: String,
}

/// A type represent chapter, normalized across source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chapter {
    pub source_id: i64,
    pub title: String,
    pub path: String,
    pub number: f64,
    pub scanlator: String,
    pub uploaded: chrono::NaiveDateTime,
}

/// Model to login to source that require login, like mangadex to search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceLogin {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub two_factor: Option<String>,
}

/// Result of source login
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceLoginResult {
    pub source_name: String,
    pub auth_type: String,
    pub value: String,
}

/// A type represent sort parameter for query manga from source, normalized across source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortByParam {
    LastUpdated,
    Title,
    Comment,
    Views,
}

impl Default for SortByParam {
    fn default() -> Self {
        SortByParam::Title
    }
}

/// A type represent order parameter for query manga from source, normalized across source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortOrderParam {
    Asc,
    Desc,
}

impl Default for SortOrderParam {
    fn default() -> Self {
        SortOrderParam::Asc
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub keyword: Option<String>,
    pub genres: Option<Vec<String>>,
    pub page: Option<i32>,
    pub sort_by: Option<SortByParam>,
    pub sort_order: Option<SortOrderParam>,
    pub auth: Option<String>,
}

impl Default for Param {
    fn default() -> Self {
        Param {
            keyword: None,
            genres: None,
            page: Some(1),
            sort_by: Some(SortByParam::Views),
            sort_order: Some(SortOrderParam::Desc),
            auth: None,
        }
    }
}

pub type ParamFilterValue = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filters {
    pub default: String,
    pub fields: BTreeMap<String, FilterField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterField {
    pub name: String,
    pub values: Option<Vec<FilterValue>>,
    #[serde(default)]
    pub multi: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterValue {
    pub title: String,
    pub value: Option<String>,
    pub related: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionResult<T> {
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Clone> ExtensionResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: &str) -> Self {
        Self {
            data: None,
            error: Some(msg.to_string()),
        }
    }

    pub fn result(&self) -> Result<T, Box<dyn std::error::Error>> {
        if let Some(data) = &self.data {
            Ok(data.clone())
        } else if let Some(err) = &self.error {
            Err(err.clone().into())
        } else {
            Err("neither data or error exists".into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let version1 = Version::from_str("0.0.0").unwrap();
        let version2 = Version::from_str("0.0.1").unwrap();

        assert!(version1 < version2);

        let version1 = Version::from_str("0.0.2").unwrap();
        let version2 = Version::from_str("0.1.1").unwrap();

        assert!(version1 < version2);

        let version1 = Version::from_str("0.1.2").unwrap();
        let version2 = Version::from_str("0.1.3").unwrap();

        assert!(version1 < version2);

        let version1 = Version::from_str("0.1.3").unwrap();
        let version2 = Version::from_str("1.1.3").unwrap();

        assert!(version1 < version2);

        let version1 = Version::from_str("1.1.2").unwrap();
        let version2 = Version::from_str("1.1.3").unwrap();

        assert!(version1 < version2);

        let version1 = Version::from_str("1.1.3").unwrap();
        let version2 = Version::from_str("1.1.3").unwrap();

        assert!(version1 == version2);
    }

    #[test]
    fn test_version_from_str() {
        let version = Version::from_str("1.2.3").unwrap();

        assert_eq!(
            version,
            Version {
                major: 1,
                minor: 2,
                patch: 3
            }
        );
    }

    #[test]
    fn test_invalid_version_from_str() {
        let version = Version::from_str("1.2.3.4");

        assert_eq!(version, Err(crate::error::Error::InvalidVersion));

        let version = Version::from_str("x.2.3");

        assert_eq!(version, Err(crate::error::Error::InvalidVersion));
    }

    #[test]
    fn test_parse_version_from_str() {
        let source = ron::from_str::<Source>(
            r#"
            Source(
                id: 0,
                name: "", 
                url: "", 
                version: "1.2.3", 
                lib_version: "1.2.3",
                icon: "", 
                need_login: false,
                languages: []
            )"#,
        );

        assert_eq!(
            source,
            Ok(Source {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3
                },
                lib_version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3
                },
                ..Default::default()
            })
        );
    }
}
