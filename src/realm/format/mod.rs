//! Realm binary format parsing and validation.
//!
//! This module provides low-level parsing of Realm file structures for validation
//! and corruption detection. Primary object access uses realm-codec (v9) or custom
//! parsers (v24+).

pub mod header;
pub mod types;
pub mod array;
pub mod schema;
pub mod deserializer;
pub mod serializer;

pub use header::{RealmHeader, FormatVersion};
pub use types::{RealmRef, TaggedValue, RefOrTagged};
pub use array::{ArrayHeader, ArrayView};
pub use schema::{Schema, TableDef, ColumnDef, ColumnType};
pub use deserializer::{Deserializer, RealmObject, Value};
pub use serializer::Serializer;
