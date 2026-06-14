use super::Value;

/// An ordered sequence of named [`Value`]s representing a document.
///
/// `ValueObject` is the named counterpart to [`ValueRecord`](super::ValueRecord):
/// where a record is positional, an object carries a key for each field. The
/// query engine builds a `ValueObject` from a positional record (using the
/// field names from the embedded model schema) just before handing
/// a document-stored value to a driver, and converts back the other way when
/// decoding driver results. Drivers serialize a `ValueObject` structurally —
/// to a JSON object, a BSON sub-document, a DynamoDB map — without needing the
/// schema.
///
/// Entries are kept in insertion order. Keys are not deduplicated; the engine
/// always builds objects from a schema, so keys are unique by construction.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ValueObject {
    /// The named field values, in insertion order.
    pub entries: Vec<(String, Value)>,

    /// Whether JSON encoding should omit entries whose value is [`Value::Null`].
    pub omit_nulls: bool,
}

impl ValueObject {
    /// Creates a schema-backed `ValueObject` from a vector of `(key, value)` pairs.
    pub fn from_vec(entries: Vec<(String, Value)>) -> Self {
        Self {
            entries,
            omit_nulls: true,
        }
    }

    /// Creates a dynamic JSON `ValueObject` that preserves explicit null entries.
    pub fn from_json_vec(entries: Vec<(String, Value)>) -> Self {
        Self {
            entries,
            omit_nulls: false,
        }
    }

    /// Iterates over the `(key, value)` entries in insertion order.
    pub fn iter(&self) -> std::slice::Iter<'_, (String, Value)> {
        self.entries.iter()
    }
}

impl From<ValueObject> for Value {
    fn from(value: ValueObject) -> Self {
        Self::Object(value)
    }
}
