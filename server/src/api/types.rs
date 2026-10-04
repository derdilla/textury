use std::{error::Error, fmt::{Display}};

use serde::{Deserialize, Serialize};
use sqlx::Encode;

use crate::{compilation};

include!(concat!(env!("OUT_DIR"), "/api_types.rs"));

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct UserId(GenericId<'u'>);
impl UserId {
    pub fn random() -> UserId {
        UserId(GenericId::random())
    }

    pub fn to_string(&self) -> String { self.0.to_string() }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct ProjectId(GenericId<'p'>);
impl ProjectId {
    pub fn random() -> ProjectId {
        ProjectId(GenericId::random())
    }

    pub fn to_string(&self) -> String { self.0.to_string() }
}

impl TryFrom<&str> for UserId {
    type Error = IDDecodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(UserId(GenericId::try_from(value)?))
    }

}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(Debug)]
pub enum IDDecodeError {
    WrongLength,
    WrongFormat,
    AlphabetError,
    /// Syntactically valid ID, but doesn't fit in a i64. We wouldn't generate this
    TooHigh,
    WrongKind,
}

impl Display for IDDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to decode id: {:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct GenericId<const C: char>(u64);

impl<const C: char> GenericId<C> {
    pub fn random() -> GenericId<C> {
        GenericId(rand::random_range(0..base58::MAX))
    }

    pub fn to_string(&self) -> String {
        let value = base58::encode(self.0)
            .expect("GenericId out of range allowed by constructors");

        let fmt = format_args!(
            "{}-{}-{}",
            C,
            str::from_utf8(&value[..5]).unwrap(),
            str::from_utf8(&value[5..]).unwrap()
        );
        fmt.to_string()
    }
}

impl<const C: char> TryFrom<&str> for GenericId<C> {
    type Error = IDDecodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let re = regex::regex!(r"^(\w)-([1-9A-HJ-NP-Za-km-z]{5})-([1-9A-HJ-NP-Za-km-z]{5})$");
        let Some(captures) = re.captures(&value) else {
            return Err(IDDecodeError::WrongFormat);
        };
        let (_, [c, high, low]) = captures.extract();
        let c = c.chars().into_iter().next().unwrap();
        if c != C {
            return Err(IDDecodeError::WrongKind);
        }
        let high = base58::decode(high.chars())?;
        let low = base58::decode(low.chars())?;
        Ok(GenericId(high * (base58::BASE as u64).pow(5) + low))
    }
}

impl<const C: char> serde::Serialize for GenericId<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, const C: char> ::serde::Deserialize<'de> for GenericId<C> {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        GenericId::try_from(encoded.as_str())
            .map_err(serde::de::Error::custom)
    }
}

impl<'r, DB: sqlx::Database, const C: char> sqlx::Type<DB> for GenericId<C>
where
    i64: sqlx::Type<DB>
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<DB>>::type_info()
    }

    fn compatible(ty: &<DB as sqlx::Database>::TypeInfo) -> bool {
        <i64 as sqlx::Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: sqlx::Database, const C: char> sqlx::Decode<'r, DB> for GenericId<C>
where
    i64: sqlx::Decode<'r, DB>
{
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'r>,
    ) -> Result<GenericId<C>, Box<dyn Error + 'static + Send + Sync>> {
        let value = <i64 as sqlx::Decode<DB>>::decode(value)?;

        Ok(GenericId(value.cast_unsigned()))
    }
}

impl<'r, DB: sqlx::Database, const C: char> Encode<'r, DB> for GenericId<C>
where
    i64: sqlx::Encode<'r, DB>
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    {
        self.0.cast_signed().encode_by_ref(buf)
    }
}

mod base58 {
    use crate::api::types::IDDecodeError;

    const ID_ALPHABET: &'static [u8; BASE] =
        b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    /// Reversed alphabet; 0xFF for undefined characters, 0-57 for defined ones.
    const ID_LOOKUP: [u8; 256] = build_lookup();

    pub const BASE: usize = 58;
    pub const ID_LENGTH: usize = 10;
    pub const MAX: u64 = (BASE as u64).pow(ID_LENGTH as u32) - 1;

    const fn build_lookup() -> [u8; 256] {
        let mut table = [0xFFu8; 256];
        let mut index = 0;
        while index < ID_ALPHABET.len() {
            table[ID_ALPHABET[index] as usize] = index as u8;
            index += 1;
        }
        table
    }

    pub fn encode(mut value: u64) -> Option<[u8; ID_LENGTH]> {
        if value >= MAX {
            return None;
        }

        let mut output = [b"1"[0] as u8; ID_LENGTH];

        let mut index = 0;
        while value != 0 {
            let remainder = (value % BASE as u64) as usize;
            output[(ID_LENGTH - 1) - index] = ID_ALPHABET[remainder];
            index += 1;
            value = value / BASE as u64;
        }

        Some(output)
    }

    pub fn decode<'a>(chars: std::str::Chars<'a>) -> Result<u64, IDDecodeError> {
        let mut result = 0;
        let mut i = 0;
        for c in chars {
            if i > 10 {
                return Err(IDDecodeError::TooHigh);
            }

            if c as usize >= 256 {
                return Err(IDDecodeError::AlphabetError);
            }
            let value = ID_LOOKUP[c as usize];
            if value == 0xFF {
                return Err(IDDecodeError::AlphabetError);
            }

            result = (BASE as u64) * result + (value as u64);

            i += 1;
        }
        Ok(result)
    }

    #[cfg(test)]
    mod tests {
        use proptest::prelude::*;
        use crate::api::types::base58;

        #[test]
        fn can_encode() {
            let enc = base58::encode(1234567890).unwrap();
            assert_eq!(str::from_utf8(&enc).unwrap(), "11112t6V2H");
        }

        #[test]
        fn can_decode() {
            let res = base58::decode("2t6V2H".chars()).unwrap();
            assert_eq!(res, 1234567890);
            let res = base58::decode("111112t6V2H".chars()).unwrap();
            assert_eq!(res, 1234567890);
        }

        #[test]
        fn doesnt_crash_big_numbers() {
            let res = base58::decode("HAHAHHAHHHAHAHAHAHAHAH".chars());
            assert!(res.is_err());
        }

        proptest! {
            #[test]
            fn doesnt_crash(s in ".*") {
                _ = base58::decode(s.chars());
            }

            #[test]
            fn parses_valid(s in "[1-9A-HJ-NP-Za-km-z]{1,10}") {
                let res = base58::decode(s.chars());
                assert!(res.is_ok())
            }

            #[test]
            fn roundtrip_works(i in u64::MIN..=base58::MAX) {
                let raw_enc = base58::encode(i).unwrap();
                let enc = str::from_utf8(&raw_enc).unwrap();
                let res = base58::decode(enc.chars()).unwrap();
                assert_eq!(res, i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

use crate::api::types::{GenericId, base58};

    #[test]
    fn can_encode() {
        let id = GenericId::<'u'>(1234567890);
        let res = serde_json::to_string(&id).unwrap();
        assert_eq!(res, "\"u-11112-t6V2H\"");
    }

    #[test]
    fn can_decode() {
        let id: GenericId<'u'> = serde_json::from_str("\"u-11112-t6V2H\"").unwrap();
        assert_eq!(id.0, 1234567890);

        let id: GenericId<'p'> = serde_json::from_str("\"p-11112-t6V2J\"").unwrap();
        assert_eq!(id.0, 1234567891);
    }

    #[test]
    fn doesnt_decode_wrong_prefix() {
        let id = serde_json::from_str::<GenericId<'u'>>("\"p-11112-t6V2H\"");
        assert!(id.is_err());
    }

    #[test]
    fn doesnt_crash_big_numbers() {
        let id: GenericId<'u'> = serde_json::from_str("\"u-zzzzz-zzzzz\"").unwrap();
        assert_eq!(id.0, base58::MAX);
    }

    proptest! {
        #[test]
        fn doesnt_crash(s in "g.*") {
            let _id: Option<GenericId<'g'>> = serde_json::from_str(&s).ok();
        }

        #[test]
        fn parses_valid(s in "\"g-([1-9A-HJ-NP-Za-km-z]{5})-([1-9A-HJ-NP-Za-km-z]{5})\"") {
            let id = serde_json::from_str::<GenericId<'g'>>(&s);
            assert!(id.is_ok())
        }

        #[test]
        fn roundtrip_works(i in u64::MIN..=base58::MAX) {
            let id = GenericId::<'g'>(i);
            let enc = serde_json::to_string(&id).unwrap();
            let res: Result<GenericId<'g'>, _> = serde_json::from_str(&enc);

            assert_eq!(res.unwrap(), id);
        }
    }
}


impl Into<compilation::Compiler> for Compiler {
    fn into(self) -> compilation::Compiler {
        match self {
            Compiler::PdfLatex => compilation::Compiler::PdfLatex,
            Compiler::XeLatex => compilation::Compiler::XeLatex,
            Compiler::LuaLatex => compilation::Compiler::LuaLatex,
            Compiler::Typst => compilation::Compiler::Typst,
        }
    }
}
