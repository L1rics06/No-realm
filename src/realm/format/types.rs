//! Core binary format types with safe abstractions.
//!
//! These types provide type-safe wrappers around raw u64 values used in Realm's
//! binary format, preventing common errors like treating offsets as values.

/// Type-safe file offset reference.
///
/// Realm file offsets must be:
/// - 8-byte aligned (LSB = 0)
/// - Within file bounds
///
/// This newtype prevents accidental arithmetic on offsets and enforces alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RealmRef(u64);

impl RealmRef {
    /// Create a RealmRef from a raw u64 value.
    ///
    /// # Errors
    ///
    /// Returns `None` if the value is not 8-byte aligned (LSB != 0).
    pub fn new(value: u64) -> Option<Self> {
        if (value & 0x01) == 0 {
            Some(RealmRef(value))
        } else {
            None
        }
    }

    /// Create a RealmRef without alignment checking.
    ///
    /// # Safety
    ///
    /// Caller must ensure the value is 8-byte aligned.
    /// Use `new()` unless you've already validated alignment.
    pub const fn new_unchecked(value: u64) -> Self {
        RealmRef(value)
    }

    /// Get the raw offset value.
    pub const fn offset(&self) -> u64 {
        self.0
    }

    /// Convert to usize for indexing, checking bounds.
    ///
    /// # Errors
    ///
    /// Returns `None` if:
    /// - Offset exceeds usize::MAX (on 32-bit platforms)
    /// - Offset is greater than or equal to file_size
    pub fn checked_deref(&self, file_size: usize) -> Option<usize> {
        let offset = usize::try_from(self.0).ok()?;
        if offset < file_size {
            Some(offset)
        } else {
            None
        }
    }

    /// Check if this is a valid reference (8-byte aligned).
    pub const fn is_valid(&self) -> bool {
        (self.0 & 0x01) == 0
    }

    /// Check if this reference is null (zero).
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl From<RealmRef> for u64 {
    fn from(r: RealmRef) -> u64 {
        r.0
    }
}

/// Type-safe tagged inline value.
///
/// Realm uses a ref-or-tagged encoding:
/// - If LSB = 0: it's a RealmRef (file offset)
/// - If LSB = 1: it's a TaggedValue (inline integer, value = raw >> 1)
///
/// This is used for small integers that don't need separate storage:
/// - Tree depths
/// - Compact leaf counts
/// - Link target table IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaggedValue(u64);

impl TaggedValue {
    /// Create a TaggedValue from a raw u64.
    ///
    /// # Errors
    ///
    /// Returns `None` if LSB is not set (not a tagged value).
    pub fn new(raw: u64) -> Option<Self> {
        if (raw & 0x01) == 1 {
            Some(TaggedValue(raw))
        } else {
            None
        }
    }

    /// Create a TaggedValue from a decoded integer value.
    ///
    /// The value is encoded as (value << 1) | 1.
    pub fn from_value(value: i64) -> Self {
        let raw = ((value as u64) << 1) | 1;
        TaggedValue(raw)
    }

    /// Get the decoded integer value.
    ///
    /// Decodes by: value = raw >> 1 (with sign extension for negative values).
    pub fn value(&self) -> i64 {
        // Right shift as signed to preserve sign bit
        ((self.0 as i64) >> 1)
    }

    /// Get the raw encoded value.
    pub const fn raw(&self) -> u64 {
        self.0
    }

    /// Check if a raw u64 is a tagged value (LSB = 1).
    pub const fn is_tagged(raw: u64) -> bool {
        (raw & 0x01) == 1
    }
}

impl From<TaggedValue> for i64 {
    fn from(t: TaggedValue) -> i64 {
        t.value()
    }
}

impl From<TaggedValue> for u64 {
    fn from(t: TaggedValue) -> u64 {
        t.0
    }
}

/// Discriminate between RealmRef and TaggedValue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefOrTagged {
    /// A file offset reference (LSB = 0).
    Ref(RealmRef),
    /// An inline tagged value (LSB = 1).
    Tagged(TaggedValue),
}

impl RefOrTagged {
    /// Parse a raw u64 into RefOrTagged.
    pub fn parse(raw: u64) -> Self {
        if (raw & 0x01) == 0 {
            RefOrTagged::Ref(RealmRef(raw))
        } else {
            RefOrTagged::Tagged(TaggedValue(raw))
        }
    }

    /// Check if this is a reference.
    pub const fn is_ref(&self) -> bool {
        matches!(self, RefOrTagged::Ref(_))
    }

    /// Check if this is a tagged value.
    pub const fn is_tagged(&self) -> bool {
        matches!(self, RefOrTagged::Tagged(_))
    }

    /// Get the reference if this is a Ref variant.
    pub fn as_ref(&self) -> Option<RealmRef> {
        match self {
            RefOrTagged::Ref(r) => Some(*r),
            _ => None,
        }
    }

    /// Get the tagged value if this is a Tagged variant.
    pub fn as_tagged(&self) -> Option<TaggedValue> {
        match self {
            RefOrTagged::Tagged(t) => Some(*t),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_ref_aligned() {
        assert!(RealmRef::new(0x1000).is_some());
        assert!(RealmRef::new(0x2000).is_some());
        assert!(RealmRef::new(0).is_some());
    }

    #[test]
    fn realm_ref_unaligned() {
        assert!(RealmRef::new(0x1001).is_none());
        assert!(RealmRef::new(0x1003).is_none());
        assert!(RealmRef::new(1).is_none());
    }

    #[test]
    fn realm_ref_offset() {
        let r = RealmRef::new(0x1234).unwrap();
        assert_eq!(r.offset(), 0x1234);
    }

    #[test]
    fn realm_ref_checked_deref_valid() {
        let r = RealmRef::new(0x100).unwrap();
        assert_eq!(r.checked_deref(0x200), Some(0x100));
    }

    #[test]
    fn realm_ref_checked_deref_out_of_bounds() {
        let r = RealmRef::new(0x200).unwrap();
        assert_eq!(r.checked_deref(0x100), None);
    }

    #[test]
    fn realm_ref_is_null() {
        let r = RealmRef::new(0).unwrap();
        assert!(r.is_null());

        let r = RealmRef::new(0x100).unwrap();
        assert!(!r.is_null());
    }

    #[test]
    fn tagged_value_from_raw() {
        let t = TaggedValue::new(0b1101).unwrap(); // LSB = 1
        assert_eq!(t.value(), 0b110); // raw >> 1
    }

    #[test]
    fn tagged_value_from_raw_not_tagged() {
        assert!(TaggedValue::new(0b1100).is_none()); // LSB = 0
    }

    #[test]
    fn tagged_value_from_value() {
        let t = TaggedValue::from_value(42);
        assert_eq!(t.value(), 42);
        assert_eq!(t.raw(), (42 << 1) | 1);
    }

    #[test]
    fn tagged_value_zero() {
        let t = TaggedValue::from_value(0);
        assert_eq!(t.value(), 0);
        assert_eq!(t.raw(), 1); // (0 << 1) | 1
    }

    #[test]
    fn tagged_value_negative() {
        let t = TaggedValue::from_value(-5);
        assert_eq!(t.value(), -5);
    }

    #[test]
    fn tagged_value_is_tagged() {
        assert!(TaggedValue::is_tagged(0b1));
        assert!(TaggedValue::is_tagged(0b11));
        assert!(!TaggedValue::is_tagged(0b10));
        assert!(!TaggedValue::is_tagged(0));
    }

    #[test]
    fn ref_or_tagged_parse_ref() {
        let rot = RefOrTagged::parse(0x1000); // LSB = 0
        assert!(rot.is_ref());
        assert_eq!(rot.as_ref().unwrap().offset(), 0x1000);
    }

    #[test]
    fn ref_or_tagged_parse_tagged() {
        let rot = RefOrTagged::parse(0b10101); // LSB = 1
        assert!(rot.is_tagged());
        assert_eq!(rot.as_tagged().unwrap().value(), 0b1010);
    }

    #[test]
    fn ref_or_tagged_parse_zero() {
        let rot = RefOrTagged::parse(0);
        assert!(rot.is_ref());
        assert!(rot.as_ref().unwrap().is_null());
    }
}
