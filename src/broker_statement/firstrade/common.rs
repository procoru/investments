use serde::Deserialize;
use serde::de::{Deserializer, Error};

use crate::core::GenericResult;
use crate::types::{Date, Decimal};
use crate::util;

#[derive(Debug, Deserialize)]
pub struct Ignore {
}

#[derive(Debug, Deserialize)]
pub struct DecimalField {
    #[serde(rename = "$value")]
    pub value: Decimal,
}

fn parse_date(date: &str) -> GenericResult<Date> {
    let format = match date.len() {
        14 => "%Y%m%d000000",
        _ => "%Y%m%d",
    };
    util::parse_date(date, format)
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error> where D: Deserializer<'de> {
    let date: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_date(&date).map_err(D::Error::custom)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parsing() {
        assert_eq!(parse_date("20200623").unwrap(), date!(23, 6, 2020));
        assert_eq!(parse_date("20200623000000").unwrap(), date!(23, 6, 2020));
    }
}