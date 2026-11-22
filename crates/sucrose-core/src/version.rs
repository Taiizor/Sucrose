//! Versions as the C# app writes them: .NET `System.Version` semantics, four components, missing ones are -1.
//!
//! Themes carry an `AppVersion` and a `Version` in `SucroseInfo.json`; the app compares its own version with the first
//! one before it plays a theme.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;

/// A version with the four .NET components. A component that is not part of the text is -1, so `26.10.19` sorts before
/// `26.10.19.0` and identical texts compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LegacyVersion {
    /// First component.
    pub major: i32,
    /// Second component.
    pub minor: i32,
    /// Third component, -1 when absent.
    pub build: i32,
    /// Fourth component, -1 when absent.
    pub revision: i32,
}

/// Why a text or an object is not a version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VersionError {
    /// A version has two to four components.
    #[error("a version has two to four components")]
    ComponentCount,
    /// A component is empty, signed, not a number or larger than an i32.
    #[error("a version component is not a non-negative number")]
    Component,
}

impl LegacyVersion {
    /// Builds a version from its components; pass -1 for a missing build or revision.
    pub const fn new(major: i32, minor: i32, build: i32, revision: i32) -> Self {
        Self {
            major,
            minor,
            build,
            revision,
        }
    }

    /// Parses `a.b`, `a.b.c` or `a.b.c.d` like `System.Version.Parse`: digits only, missing components are -1.
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        let parts: Vec<&str> = text.trim().split('.').collect();
        if !(2..=4).contains(&parts.len()) {
            return Err(VersionError::ComponentCount);
        }
        let mut components = [-1; 4];
        for (slot, part) in components.iter_mut().zip(&parts) {
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(VersionError::Component);
            }
            *slot = part.parse().map_err(|_| VersionError::Component)?;
        }
        let [major, minor, build, revision] = components;
        Ok(Self::new(major, minor, build, revision))
    }

    /// Maps a Cargo version `major.minor.patch[+revision]` onto the four .NET components.
    pub fn from_cargo(_text: &str) -> Result<Self, VersionError> {
        Ok(Self::default())
    }
}

/// The version of this build as the C# app would write it (D-029).
pub fn app_version() -> LegacyVersion {
    LegacyVersion::default()
}

/// The original text: only the components that exist, so `parse(text)` prints back as `text`.
impl fmt::Display for LegacyVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;
        if self.build >= 0 {
            write!(f, ".{}", self.build)?;
            if self.revision >= 0 {
                write!(f, ".{}", self.revision)?;
            }
        }
        Ok(())
    }
}

impl Serialize for LegacyVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// One component of the object form: absent or null is -1, anything else must be an i32.
fn object_component(object: &serde_json::Map<String, Value>, key: &str) -> Result<i32, VersionError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(-1),
        Some(Value::Number(number)) => number
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or(VersionError::Component),
        Some(_) => Err(VersionError::Component),
    }
}

impl<'de> Deserialize<'de> for LegacyVersion {
    /// Reads the string form (`"1.0.1.1"`) or the object form (`{"Major":1,"Minor":0,"Build":1,"Revision":1}`); other
    /// properties of the object (`MajorRevision`, `MinorRevision`) are ignored.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(text) => Self::parse(&text).map_err(de::Error::custom),
            Value::Object(object) => {
                let major = object_component(&object, "Major").map_err(de::Error::custom)?;
                let minor = object_component(&object, "Minor").map_err(de::Error::custom)?;
                let build = object_component(&object, "Build").map_err(de::Error::custom)?;
                // As Newtonsoft's VersionConverter: a revision without a build is dropped.
                let revision = if build < 0 {
                    -1
                } else {
                    object_component(&object, "Revision").map_err(de::Error::custom)?
                };
                if major < 0 || minor < 0 {
                    return Err(de::Error::custom(VersionError::Component));
                }
                Ok(Self::new(major, minor, build.max(-1), revision.max(-1)))
            }
            _ => Err(de::Error::custom("a version is a string or an object")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> LegacyVersion {
        LegacyVersion::parse(text).expect("valid version")
    }

    #[test]
    fn parse_fills_missing_components_with_minus_one() {
        assert_eq!(v("1.0"), LegacyVersion::new(1, 0, -1, -1));
        assert_eq!(v("26.10.19"), LegacyVersion::new(26, 10, 19, -1));
        assert_eq!(v("26.10.19.0"), LegacyVersion::new(26, 10, 19, 0));
    }

    #[test]
    fn parse_refuses_malformed_text() {
        for text in [
            "",
            "1",
            "1.2.3.4.5",
            "1..2",
            "a.b",
            "-1.0",
            "+1.0",
            "1.0.",
            "99999999999.0",
        ] {
            assert!(LegacyVersion::parse(text).is_err(), "{text:?} must be refused");
        }
    }

    #[test]
    fn from_cargo_maps_build_metadata_to_the_revision() {
        assert_eq!(
            LegacyVersion::from_cargo("26.11.22+1"),
            Ok(LegacyVersion::new(26, 11, 22, 1))
        );
        assert_eq!(
            LegacyVersion::from_cargo("26.11.23"),
            Ok(LegacyVersion::new(26, 11, 23, 0))
        );
        assert_eq!(
            LegacyVersion::from_cargo("26.11.22+12"),
            Ok(LegacyVersion::new(26, 11, 22, 12))
        );
        for text in [
            "",
            "26.11",
            "26.11.22.1",
            "26.11.22-beta",
            "26.11.22+",
            "26.11.22+x",
            "26.011.22",
            "a.b.c",
        ] {
            assert!(LegacyVersion::from_cargo(text).is_err(), "{text:?} must be refused");
        }
    }

    #[test]
    fn the_app_is_newer_than_the_last_csharp_release() {
        assert!(app_version() > v("26.11.22.0"));
    }

    /// Assumption A16: Cargo hands build metadata through, so `CARGO_PKG_VERSION` keeps the `+1`.
    #[test]
    fn cargo_version_text_keeps_the_build_metadata() {
        assert!(crate::VERSION.contains("26.11.22+1"), "VERSION is {}", crate::VERSION);
        assert_eq!(
            app_version(),
            LegacyVersion::from_cargo(crate::VERSION).expect("workspace version")
        );
    }

    #[test]
    fn display_prints_back_the_text() {
        for text in ["1.0", "1.0.0", "26.10.19.0", "26.11.22.1"] {
            assert_eq!(v(text).to_string(), text);
        }
    }

    #[test]
    fn ordering_goes_component_by_component_with_missing_below_zero() {
        assert!(v("26.10.19") < v("26.10.19.0"));
        assert!(v("26.10.19.0") < v("26.10.19.1"));
        assert!(v("26.9.30.5") < v("26.10.1"));
        assert_eq!(v("1.0"), v("1.0"));
        assert_eq!(v("1.0").cmp(&v("1.0")), std::cmp::Ordering::Equal);
    }

    #[test]
    fn serde_reads_a_string_and_an_object() {
        let from_string: LegacyVersion = serde_json::from_str("\"1.0.1.1\"").expect("string form");
        let from_object: LegacyVersion =
            serde_json::from_str(r#"{"Major":1,"Minor":0,"Build":1,"Revision":1,"MajorRevision":0}"#)
                .expect("object form");
        let short: LegacyVersion = serde_json::from_str(r#"{"Major":1,"Minor":0}"#).expect("short object");

        assert_eq!(from_string, from_object);
        assert_eq!(short, LegacyVersion::new(1, 0, -1, -1));
        assert_eq!(serde_json::to_string(&short).expect("write"), "\"1.0\"");
    }

    #[test]
    fn serde_refuses_other_shapes() {
        for text in [
            "1",
            "null",
            "[1,0]",
            r#"{"Major":1}"#,
            r#"{"Major":-1,"Minor":0}"#,
            "\"x.y\"",
        ] {
            assert!(
                serde_json::from_str::<LegacyVersion>(text).is_err(),
                "{text} must be refused"
            );
        }
    }
}
