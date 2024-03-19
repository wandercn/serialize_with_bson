//! # serialize_with_bson
//!
//! bson DateTime 转json 帮助函数，转换指定时区和格式的时间格式
//!
//! # Example
//!
//! ```rust
//! use serialize_with_bson::{
//!     datetime_to_tz, datetime_to_tz_map, layout::DEFAULT, object_to_hex, time_zone_and_layout,
//!     TimeZoner,
//! };
//! use bson::{doc, oid::ObjectId, DateTime, Document};
//! use serde::{Deserialize, Serialize};
//! use serde_json::to_string;
//! use std::{collections::HashMap, str::FromStr};
//!  
//!     time_zone_and_layout!("Asia/Tokyo", DEFAULT,BsonDateTime); // 设定 DateTime的包装类型为BsonDateTime 设定时区和时间字符串输出格式
//!
//!     #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//!     pub struct Bacterium {
//!         #[serde(rename = "_id", serialize_with = "object_to_hex")] // 启用自定义序列化函数 object_to_hex 把ObjectID转成hex字符串
//!         pub id: ObjectId,
//!         pub has_genome: bool,
//!         #[serde(serialize_with = "datetime_to_tz")] // 启用自定义序列化函数datetime_to_tz 把BsonDateTime转到指定时区时间
//!         pub creation_time: BsonDateTime,
//!         #[serde(serialize_with = "datetime_to_tz")]
//!         pub modified_time: BsonDateTime,
//!         pub short_id: String,
//!         #[serde(serialize_with = "datetime_to_tz_map")] // 启用自定义序列化函数datetime_to_tz_map 把Map中的BsonDateTime转到指定时区时间
//!         pub locations: HashMap<String, BsonDateTime>,
//!         pub taxonomy: Document,
//!         pub backtrace: Vec<String>,
//!     }
//!
//!     
//! fn main() {
//!         let mut map: HashMap<String, BsonDateTime> = HashMap::new();
//!         map.insert(
//!             "R3R-A-9-2-L5".to_string(),
//!             DateTime::from_millis(1571985978429).into(),
//!         );
//!         map.insert(
//!             "R3T-A-5-3-K6".to_string(),
//!             BsonDateTime::from(DateTime::from_millis(1571984742668)),
//!         );
//!         map.insert(
//!             "R3R-A-10-3-L7".to_string(),
//!             BsonDateTime::from_millis(1571985444876),
//!         );
//!
//!         let src = Bacterium {
//!             id: ObjectId::from_str("5db131829181e500010b93d6").unwrap(),
//!             has_genome: false,
//!             creation_time: DateTime::from_millis(1571893634109).into(),
//!             modified_time: DateTime::from_millis(1690439186944).into(),
//!             short_id: "B1DXX".to_owned(),
//!             locations: map,
//!             taxonomy: doc! {
//!               "class": "Bacilli",
//!               "phylum": "Bacillota",
//!               "kingdom": "Bacteria",
//!               "cnSpecies": "粪肠球菌",
//!               "species": "Enterococcus faecalis",
//!               "genus": "Enterococcus",
//!               "family": "Enterococcaceae",
//!               "order": "Lactobacillales"
//!             },
//!             backtrace: vec!["H2T73".to_owned(), "H2RNV".to_owned()],
//!         };
//!
//!         if let Ok(result) = to_string(&src) {
//!             println!("{}", result);
//!         }
//!     }
//! ```
//! #Output:
//! ```json
//!  {
//! 	"_id": "5db131829181e500010b93d6",
//! 	"has_genome": false,
//! 	"creation_time": "2019-10-24 14:07:14.109 +0900 JST",
//! 	"modified_time": "2023-07-27 15:26:26.944 +0900 JST",
//! 	"short_id": "B1DXX",
//! 	"locations": {
//! 		"R3T-A-5-3-K6": "2019-10-25 15:25:42.668 +0900 JST",
//! 		"R3R-A-9-2-L5": "2019-10-25 15:46:18.429 +0900 JST",
//! },
//! 	"taxonomy": {
//! 		"class": "Bacilli",
//! 		"phylum": "Bacillota",
//! 		"kingdom": "Bacteria",
//! 		"cnSpecies": "粪肠球菌",
//! 		"species": "Enterococcus faecalis",
//! 		"genus": "Enterococcus",
//! 		"family": "Enterococcaceae",
//! 		"order": "Lactobacillales"
//! 	},
//! 	"backtrace": ["H2T73", "H2RNV"]
//! }
//! ```
pub mod layout;
use bson::oid::ObjectId;
use bson::DateTime;
use gostd_time::{LoadLocation, UnixMilli};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 指定时区特征
pub trait TimeZoner {
    fn tz_name(&self) -> &str;
    fn timestamp(&self) -> i64;
    fn layout(&self) -> &str;
}

/// 时间戳转指定时区时间字符串
pub fn datetime_to_tz<V, S>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize + TimeZoner,
{
    let formatted = convert_time_zone(value.timestamp(), value.tz_name(), value.layout());
    serializer.serialize_str(&formatted)
}

fn convert_time_zone(timestamp: i64, tz_name: &str, layout: &str) -> String {
    let mut t = UnixMilli(timestamp);
    let cst_zone = LoadLocation(tz_name)
        .ok()
        .expect(&format!("time_zone_name {} is not found!", tz_name));

    t.In(cst_zone);
    if layout.is_empty() {
        return t.String();
    }
    t.Format(layout)
}
/// ObjectId 转 hex 字符串
pub fn object_to_hex<S>(value: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = value.to_hex();
    serializer.serialize_str(&formatted)
}

/// map中V转指定时区字符串
pub fn datetime_to_tz_map<K, S, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize,
    S: Serializer,
    V: TimeZoner,
{
    let mut map_serializer = serializer.serialize_map(Some(value.len()))?;
    for (key, value) in value {
        map_serializer.serialize_key(key)?;
        map_serializer.serialize_value(&convert_time_zone(
            value.timestamp(),
            value.tz_name(),
            value.layout(),
        ))?;
    }
    map_serializer.end()
}

/// 设置时区和输出日期时间输出格式的宏
#[macro_export]
macro_rules! time_zone_and_layout {
    ( $tz_name:tt, $layout:ident,$new_type:ident) => {
        #[derive(
            Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize, Debug,
        )]
        pub struct $new_type(DateTime);
        /// Make a new [`BsonDateTime`] from [`DateTime`]
        impl From<DateTime> for $new_type {
            fn from(date: DateTime) -> Self {
                Self(date)
            }
        }
        impl $new_type {
            pub fn timestamp_millis(&self) -> i64 {
                self.0.timestamp_millis()
            }
            /// Makes a new [`BsonDateTime`] from the number of non-leap milliseconds since
            /// January 1, 1970 0:00:00 UTC (aka "UNIX timestamp").
            pub const fn from_millis(unix_time_stamp: i64) -> Self {
                Self(DateTime::from_millis(unix_time_stamp))
            }
            /// Returns a [`BsonDateTime`] which corresponds to the current date and time.
            pub fn now() -> Self {
                Self::from(DateTime::now())
            }
        }

        impl From<$new_type> for DateTime {
            fn from(bson_date_time: $new_type) -> Self {
                bson_date_time.0
            }
        }
        impl TimeZoner for $new_type {
            fn tz_name(&self) -> &str {
                return $tz_name;
            }

            fn timestamp(&self) -> i64 {
                self.timestamp_millis()
            }
            fn layout(&self) -> &str {
                return $layout;
            }
        }
    };
}

#[cfg(test)]
mod tests {

    use crate::{datetime_to_tz, datetime_to_tz_map, layout::DEFAULT, object_to_hex, TimeZoner};
    use bson::{doc, oid::ObjectId, DateTime, Document};
    use serde::{Deserialize, Serialize};
    use serde_json::to_string;
    use std::{collections::HashMap, convert::From, str::FromStr};

    time_zone_and_layout!("Asia/Tokyo", DEFAULT, BsonDateTime);
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Bacterium {
        #[serde(rename = "_id", serialize_with = "object_to_hex")]
        pub id: ObjectId,
        pub has_genome: bool,
        #[serde(serialize_with = "datetime_to_tz")]
        pub creation_time: BsonDateTime,
        #[serde(serialize_with = "datetime_to_tz")]
        pub modified_time: BsonDateTime,
        pub short_id: String,
        #[serde(serialize_with = "datetime_to_tz_map")]
        pub locations: HashMap<String, BsonDateTime>,
        pub taxonomy: Document,
        pub backtrace: Vec<String>,
    }

    #[test]
    fn test_convert_to_json() {
        let mut map: HashMap<String, BsonDateTime> = HashMap::new();
        map.insert(
            "R3R-A-9-2-L5".to_string(),
            DateTime::from_millis(1571985978429).into(),
        );
        map.insert(
            "R3T-A-5-3-K6".to_string(),
            BsonDateTime::from(DateTime::from_millis(1571984742668)),
        );
        map.insert(
            "R3R-A-10-3-L7".to_string(),
            BsonDateTime::from_millis(1571985444876),
        );

        let src = Bacterium {
            id: ObjectId::from_str("5db131829181e500010b93d6").unwrap(),
            has_genome: false,
            creation_time: DateTime::from_millis(1571893634109).into(),
            modified_time: DateTime::from_millis(1690439186944).into(),
            short_id: "B1DXX".to_owned(),
            locations: map,
            taxonomy: doc! {
              "class": "Bacilli",
              "phylum": "Bacillota",
              "kingdom": "Bacteria",
              "cnSpecies": "粪肠球菌",
              "species": "Enterococcus faecalis",
              "genus": "Enterococcus",
              "family": "Enterococcaceae",
              "order": "Lactobacillales"
            },
            backtrace: vec!["H2T73".to_owned(), "H2RNV".to_owned()],
        };
        if let Ok(result) = to_string(&src) {
            assert_eq!(true, true);
            println!("{:?}", src);
            println!("{}", result);
        }
    }
}
