/// Provides serialization and deserialization functions which should use a default 
/// string instead of `null` on the wire.
macro_rules! optional_string {
    ($n:ident : $s:tt) => {
        pub mod $n {
            use serde::de::value::ValueDeserializer;
            use serde::{Serialize, Serializer, Deserialize, Deserializer};

            /// The string which represents `null` on the wire.
            pub static DEFAULT: &'static str = $s;

            /// Serialize an optional value, replacing `None` with the module-specified default.
            pub fn serialize<S: Serializer, T: Serialize>(opt: &Option<T>, s: S) -> Result<S::Ok, S::Error> {
                match *opt {
                    None => s.serialize_str(DEFAULT),
                    Some(ref value) => s.serialize_some(value),
                }
            }

            /// Deserialize a string, replacing the module-specified default with `None`.
            pub fn deserialize<D: Deserializer, T: Deserialize>(d: D) -> Result<Option<T>, D::Error> {
                
                let s = String::deserialize(d)?;
                if s == DEFAULT {
                    Ok(None)
                } else {
                    T::deserialize(s.into_deserializer()).map(Some)
                }
            }
        }
    }
}

/// Generate a `FromStr` implementation which uses serde's value deserializer.
/// 
/// # Usage
/// For an already-defined type, add deserialization with `fromstr_deserialize!(Type)`. 
/// This will create an implementation of `std::str::FromStr` where `Err=String`, containing
/// the error message returned by the deserializer.
///
/// To create a deserialization that uses a custom error type, instead write
/// `fromstr_deserialize!(Type, Err = YourError)`. This requires 
/// `YourError: From<::serde::de::value::Error>`.
macro_rules! fromstr_deserialize {
    ($t:ident) => {
        impl ::std::str::FromStr for $t {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use std::string::ToString;

                use serde::Deserialize;
                use serde::de::value::{self, ValueDeserializer};

                $t::deserialize(s.into_deserializer()).map_err(|e: value::Error| e.to_string())
            }
        }
    };
    
    ($t:ident, Err = $err:ident) => {
        impl ::std::str::FromStr for $t {
            type Err = $err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use serde::Deserialize;
                use serde::de::value::{self, ValueDeserializer};

                $t::deserialize(s.into_deserializer()).map_err(|e: value::Error| $err::from(e))
            }
        }
    }
}