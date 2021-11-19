// Copyright 2021 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
//! PKCS11 General Data Types

use crate::error::{Error, Result};
use cryptoki_sys::*;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;

pub(crate) trait Flags: std::fmt::Display {
    type FlagType: Copy;

    fn flag_value(&self) -> Self::FlagType;

    fn flag(&self, flag: Self::FlagType) -> bool;

    fn set_flag(&mut self, flag: Self::FlagType, b: bool);

    fn stringify_flag(flag: Self::FlagType) -> &'static str;

    fn stringify_fmt(&self, f: &mut Formatter<'_>, flags: Vec<Self::FlagType>) -> std::fmt::Result {
        let mut first_done = false;
        for flag in flags.iter() {
            if self.flag(*flag) {
                if first_done {
                    write!(f, ", ")?;
                }
                write!(f, "{}", Self::stringify_flag(*flag))?;
                first_done = true;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
/// Value that represents a date
pub struct Date {
    date: CK_DATE,
}

impl Date {
    /// Creates a new `Date` structure
    ///
    /// # Arguments
    ///
    /// * `year` - A 4 character length year, e.g. "2021"
    /// * `month` - A 2 character length mont, e.g. "02"
    /// * `day` - A 2 character length day, e.g. "15"
    ///
    /// # Errors
    ///
    /// If the lengths are invalid a `Error::InvalidValue` will be returned
    pub fn new_from_str_slice(year: &str, month: &str, day: &str) -> Result<Self> {
        if year.len() != 4 || month.len() != 2 || day.len() != 2 {
            Err(Error::InvalidValue)
        } else {
            let mut year_slice: [u8; 4] = Default::default();
            let mut month_slice: [u8; 2] = Default::default();
            let mut day_slice: [u8; 2] = Default::default();
            year_slice.copy_from_slice(year.as_bytes());
            month_slice.copy_from_slice(month.as_bytes());
            day_slice.copy_from_slice(day.as_bytes());
            Ok(Date::new(year_slice, month_slice, day_slice))
        }
    }

    /// Creates a new `Date` structure from byte slices
    ///
    /// # Arguments
    ///
    /// * `year` - A 4 character length year, e.g. "2021"
    /// * `month` - A 2 character length mont, e.g. "02"
    /// * `day` - A 2 character length day, e.g. "15"
    pub fn new(year: [u8; 4], month: [u8; 2], day: [u8; 2]) -> Self {
        let date = CK_DATE { year, month, day };
        Self { date }
    }
}

impl Deref for Date {
    type Target = CK_DATE;

    fn deref(&self) -> &Self::Target {
        &self.date
    }
}

impl From<Date> for CK_DATE {
    fn from(date: Date) -> Self {
        *date
    }
}

impl From<CK_DATE> for Date {
    fn from(date: CK_DATE) -> Self {
        Self { date }
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let year = String::from_utf8_lossy(Vec::from(self.year).as_slice())
            .trim_end()
            .to_string();
        let month = String::from_utf8_lossy(Vec::from(self.month).as_slice())
            .trim_end()
            .to_string();
        let day = String::from_utf8_lossy(Vec::from(self.day).as_slice())
            .trim_end()
            .to_string();

        write!(f, "Month: {}\nDay: {}\nYear: {}", month, day, year)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
/// Unsigned value, at least 32 bits long
pub struct Ulong {
    val: CK_ULONG,
}

impl Deref for Ulong {
    type Target = CK_ULONG;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl From<Ulong> for CK_ULONG {
    fn from(ulong: Ulong) -> Self {
        *ulong
    }
}

impl From<CK_ULONG> for Ulong {
    fn from(ulong: CK_ULONG) -> Self {
        Ulong { val: ulong }
    }
}

impl TryFrom<usize> for Ulong {
    type Error = Error;

    fn try_from(ulong: usize) -> Result<Self> {
        Ok(Ulong {
            val: ulong.try_into()?,
        })
    }
}

impl From<Ulong> for usize {
    fn from(ulong: Ulong) -> Self {
        ulong.val as usize
    }
}

impl std::fmt::Display for Ulong {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

pub(crate) trait MaybeUnavailable: Sized {
    fn maybe_unavailable(value: CK_ULONG) -> Option<Self>;
}

impl MaybeUnavailable for usize {
    fn maybe_unavailable(value: CK_ULONG) -> Option<usize> {
        if value == CK_UNAVAILABLE_INFORMATION {
            None
        } else {
            Some(value as usize)
        }
    }
}

impl MaybeUnavailable for u64 {
    fn maybe_unavailable(value: CK_ULONG) -> Option<u64> {
        if value == CK_UNAVAILABLE_INFORMATION {
            None
        } else {
            // Must have cast for when ulong is 32 bits
            // Must have lint suppression when ulong is 64 bits
            #[allow(trivial_numeric_casts)]
            Some(value as u64)
        }
    }
}

pub(crate) fn maybe_unlimited(value: CK_ULONG) -> Option<Option<u64>> {
    match value {
        CK_UNAVAILABLE_INFORMATION => None,
        CK_EFFECTIVELY_INFINITE => Some(None),
        // Must have cast for when ulong is 32 bits
        // Must have lint suppression when ulong is 64 bits
        #[allow(trivial_numeric_casts)]
        _ => Some(Some(value as u64)),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Represents a version
pub struct Version {
    major: CK_BYTE,
    minor: CK_BYTE,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Version {
    /// Construct a new version
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Returns the major version
    pub fn major(&self) -> CK_BYTE {
        self.major
    }

    /// Returns the minor version
    pub fn minor(&self) -> CK_BYTE {
        self.minor
    }
}

impl From<Version> for CK_VERSION {
    fn from(version: Version) -> Self {
        CK_VERSION {
            major: version.major,
            minor: version.minor,
        }
    }
}

impl From<CK_VERSION> for Version {
    fn from(version: CK_VERSION) -> Self {
        Version {
            major: version.major,
            minor: version.minor,
        }
    }
}

/// A UTC datetime returned by a token's clock if present.
#[derive(Copy, Clone, Debug)]
pub struct UtcTime {
    /// **[Conformance](crate#conformance-notes):**
    /// Guaranteed to be in range 0..=9999
    pub year: u16,
    /// **[Conformance](crate#conformance-notes):**
    /// Guaranteed to be in range 0..=99
    pub month: u8,
    /// **[Conformance](crate#conformance-notes):**
    /// Guaranteed to be in range 0..=99
    pub day: u8,
    /// **[Conformance](crate#conformance-notes):**
    /// Guaranteed to be in range 0..=99
    pub hour: u8,
    /// **[Conformance](crate#conformance-notes):**
    /// Guaranteed to be in range 0..=99
    pub minute: u8,
    /// **[Conformance](crate#conformance-notes):**
    /// Guaranteed to be in range 0..=99
    pub second: u8,
}

/// Uses ISO 8601 format for convenience, but does not guarantee
/// or enforce any part of that standard.
impl fmt::Display for UtcTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

// UTC time has the format YYYYMMDDhhmmss00 as ASCII digits
pub(crate) fn convert_utc_time(orig: [u8; 16]) -> Result<Option<UtcTime>> {
    // Note: No validaiton of these values beyond being ASCII digits
    // because PKCS#11 doesn't impose any such restrictions.
    Ok(Some(UtcTime {
        year: std::str::from_utf8(&orig[0..4])?.parse()?,
        month: std::str::from_utf8(&orig[4..6])?.parse()?,
        day: std::str::from_utf8(&orig[6..8])?.parse()?,
        hour: std::str::from_utf8(&orig[8..10])?.parse()?,
        minute: std::str::from_utf8(&orig[10..12])?.parse()?,
        second: std::str::from_utf8(&orig[12..14])?.parse()?,
    }))
}

#[cfg(test)]
mod test {

    use super::*;
    const UTC_TIME: UtcTime = UtcTime {
        year: 1970,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
    };

    #[test]
    fn utc_time_convert_good() {
        let valid: [u8; 16] = [
            0x31, 0x39, 0x37, 0x30, 0x30, 0x31, 0x30, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30,
        ];
        let valid = convert_utc_time(valid).unwrap().unwrap();
        assert_eq!(valid.year, UTC_TIME.year);
        assert_eq!(valid.month, UTC_TIME.month);
        assert_eq!(valid.day, UTC_TIME.day);
        assert_eq!(valid.hour, UTC_TIME.hour);
        assert_eq!(valid.minute, UTC_TIME.minute);
        assert_eq!(valid.second, UTC_TIME.second);
    }

    #[test]
    fn utc_time_convert_bad() {
        let invalid: [u8; 16] = [
            0x41, 0x39, 0x37, 0x30, 0x30, 0x31, 0x30, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30,
        ];
        let invalid = convert_utc_time(invalid);
        assert!(invalid.is_err());
    }

    #[test]
    fn utc_time_debug_fmt() {
        let expected = r#"UtcTime {
    year: 1970,
    month: 1,
    day: 1,
    hour: 0,
    minute: 0,
    second: 0,
}"#;
        let observed = format!("{:#?}", UTC_TIME);
        assert_eq!(observed, expected);
    }
    #[test]
    fn utc_time_display_fmt() {
        let display = format!("{}", UTC_TIME);
        assert_eq!(&display, "1970-01-01T00:00:00Z");
    }
}
