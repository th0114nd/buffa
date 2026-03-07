//! Ergonomic helpers for [`google::protobuf::Empty`](crate::google::protobuf::Empty).

#[cfg(feature = "json")]
use crate::google::protobuf::Empty;

#[cfg(feature = "json")]
impl serde::Serialize for Empty {
    /// Serializes as an empty JSON object `{}`.
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        s.serialize_map(Some(0))?.end()
    }
}

#[cfg(feature = "json")]
impl<'de> serde::Deserialize<'de> for Empty {
    /// Deserializes from a JSON object (any fields are ignored).
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::{MapAccess, Visitor};
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Empty;
            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("an empty JSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Empty, A::Error> {
                while map
                    .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
                    .is_some()
                {}
                Ok(Empty {
                    ..Default::default()
                })
            }
        }
        d.deserialize_map(V)
    }
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use super::*;

    #[test]
    fn empty_serializes_as_empty_object() {
        let e = Empty::default();
        assert_eq!(serde_json::to_string(&e).unwrap(), "{}");
    }

    #[test]
    fn empty_deserializes_from_empty_object() {
        let _: Empty = serde_json::from_str("{}").unwrap();
    }
}
