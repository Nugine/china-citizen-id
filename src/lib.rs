//! 中华人民共和国公民身份号码 (GB 11643-1999)

#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]
#![allow(
    clippy::inline_always,
    clippy::needless_range_loop,
    clippy::missing_errors_doc, // TODO
)]
// ---
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::fmt;
use core::str;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    InvalidLength,
    InvalidCharacter,
    WrongCheckNumber,
    InvalidBirthday,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl core::error::Error for Error {}

pub struct ParsedIdNumber {
    sex: Sex,
    birthday: (u16, u8, u8),
    region: Region,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

impl ParsedIdNumber {
    #[must_use]
    pub fn sex(&self) -> Sex {
        self.sex
    }

    #[must_use]
    pub fn birthday_ymd(&self) -> (u16, u8, u8) {
        self.birthday
    }

    #[must_use]
    pub fn region(&self) -> &Region {
        &self.region
    }
}

/// 二代身份证号 (18位)
pub fn parse_v2(id_str: &str) -> Result<ParsedIdNumber, Error> {
    let id: [u8; 18] = id_str
        .as_bytes()
        .try_into()
        .map_err(|_| Error::InvalidLength)?;

    for i in 0..17 {
        if !id[i].is_ascii_digit() {
            return Err(Error::InvalidCharacter);
        }
    }
    if !id[17].is_ascii_digit() && id[17] != b'X' {
        return Err(Error::InvalidCharacter);
    }

    {
        const W: [u8; 17] = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
        let mut sum: u32 = if id[17] == b'X' {
            10
        } else {
            u32::from(id[17] - b'0')
        };
        for i in 0..17 {
            sum += u32::from(id[i] - b'0') * u32::from(W[i]);
            sum %= 11;
        }
        if sum != 1 {
            return Err(Error::WrongCheckNumber);
        }
    }

    let birthday = {
        let year = u16_from_char4([id[6], id[7], id[8], id[9]]);
        let month = u8_from_char2([id[10], id[11]]);
        let day = u8_from_char2([id[12], id[13]]);

        // Please change it after I'm 200 years old :)
        if year <= 1800 || year >= 2200 {
            return Err(Error::InvalidBirthday);
        }

        if !validate_ymd(year, month, day) {
            return Err(Error::InvalidBirthday);
        }

        (year, month, day)
    };

    let region = {
        let region_code = [id[0], id[1], id[2], id[3], id[4], id[5]];
        get_region(region_code, birthday.0)
    };

    let sex = if id[16] & 1 == 1 {
        Sex::Male
    } else {
        Sex::Female
    };

    Ok(ParsedIdNumber {
        sex,
        birthday,
        region,
    })
}

/// 一代身份证号 (15位)
pub fn parse_v1(id_str: &str) -> Result<ParsedIdNumber, Error> {
    let id: [u8; 15] = id_str
        .as_bytes()
        .try_into()
        .map_err(|_| Error::InvalidLength)?;

    for i in 0..15 {
        if !id[i].is_ascii_digit() {
            return Err(Error::InvalidCharacter);
        }
    }

    let birthday = {
        let year = u16_from_char4([b'1', b'9', id[6], id[7]]);
        let month = u8_from_char2([id[8], id[9]]);
        let day = u8_from_char2([id[10], id[11]]);

        if !validate_ymd(year, month, day) {
            return Err(Error::InvalidBirthday);
        }

        (year, month, day)
    };

    let region = {
        let region_code = [id[0], id[1], id[2], id[3], id[4], id[5]];
        get_region(region_code, birthday.0)
    };

    let sex = if id[14] & 1 == 1 {
        Sex::Male
    } else {
        Sex::Female
    };

    Ok(ParsedIdNumber {
        sex,
        birthday,
        region,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub province: Option<&'static str>,
    pub city: Option<&'static str>,
    pub district: Option<&'static str>,
}

fn get_region(region_code: [u8; 6], year: u16) -> Region {
    static DATASET: LazyLock<HashMap<u16, HashMap<&'static str, &'static str>>> =
        LazyLock::new(|| serde_json::from_str(include_str!("region.json")).unwrap());

    let c = &region_code;
    let t1 = [c[0], c[1], b'0', b'0', b'0', b'0'];
    let t1 = str::from_utf8(&t1).unwrap();
    let t2 = [c[0], c[1], c[2], c[3], b'0', b'0'];
    let t2 = str::from_utf8(&t2).unwrap();
    let t3 = str::from_utf8(&region_code).unwrap();

    let dataset = &*DATASET;

    if t1 == t3 {
        if let Some(data) = dataset.get(&year) {
            let province = data.get(t1).copied();
            return Region {
                province,
                city: None,
                district: None,
            };
        }
    } else if let Some(data) = dataset.get(&year) {
        let province = data.get(t1).copied();
        let city = data.get(t2).copied();
        let district = data.get(t3).copied();
        return Region {
            province,
            city,
            district,
        };
    }

    Region {
        province: None,
        city: None,
        district: None,
    }
}

fn validate_ymd(year: u16, month: u8, day: u8) -> bool {
    let month: time::Month = match month.try_into() {
        Ok(m) => m,
        Err(_) => return false,
    };

    time::Date::from_calendar_date(i32::from(year), month, day).is_ok()
}

#[inline(always)]
fn u16_from_char4(c: [u8; 4]) -> u16 {
    u16::from(c[0] - b'0') * 1000
        + u16::from(c[1] - b'0') * 100
        + u16::from(c[2] - b'0') * 10
        + u16::from(c[3] - b'0')
}

#[inline(always)]
fn u8_from_char2(c: [u8; 2]) -> u8 {
    (c[0] - b'0') * 10 + (c[1] - b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        {
            let id = "11010519491231002X";
            let parsed = parse_v2(id).unwrap();
            assert_eq!(parsed.birthday, (1949, 12, 31));
            assert_eq!(parsed.sex(), Sex::Female);
        }

        {
            let id = "440524188001010014";
            let parsed = parse_v2(id).unwrap();
            assert_eq!(parsed.birthday, (1880, 1, 1));
            assert_eq!(parsed.sex(), Sex::Male);
        }

        {
            let id = "420111198203251029";
            let parsed = parse_v2(id).unwrap();
            assert_eq!(parsed.birthday, (1982, 3, 25));
            assert_eq!(parsed.sex(), Sex::Female);
            assert_eq!(parsed.region().province, Some("湖北省"));
            assert_eq!(parsed.region().city, Some("武汉市"));
            assert_eq!(parsed.region().district, Some("洪山区"));
        }
    }
}
