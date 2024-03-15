pub mod layout;
use bson::oid::ObjectId;
use gostd_time::{LoadLocation, UnixMilli};
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
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
    ( $( $x:tt,$y:expr )* ) => {
        $(
            impl crate::TimeZoner for bson::DateTime {
                fn tz_name(&self) -> &str {
                    return $x;
                }

                fn timestamp(&self) -> i64 {
                    self.timestamp_millis()
                }
                fn layout(&self) -> &str{
                    return $y;
                }
            }
)* };
}

#[cfg(test)]
mod tests {

    use crate::{datetime_to_tz, datetime_to_tz_map, layout::DEFAULT, object_to_hex};
    use bson::{doc, oid::ObjectId, DateTime, Document};
    use serde::{Deserialize, Serialize};
    use serde_json::to_string;
    use std::{collections::HashMap, str::FromStr};

    time_zone_and_layout!("Asia/Tokyo", DEFAULT);
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Bacterium {
        #[serde(rename = "_id", serialize_with = "object_to_hex")]
        pub id: ObjectId,
        pub has_genome: bool,
        #[serde(serialize_with = "datetime_to_tz")]
        pub creation_time: DateTime,
        #[serde(serialize_with = "datetime_to_tz")]
        pub modified_time: DateTime,
        pub short_id: String,
        #[serde(serialize_with = "datetime_to_tz_map")]
        pub locations: HashMap<String, DateTime>,
        pub taxonomy: Document,
        pub backtrace: Vec<String>,
    }

    #[test]
    fn test_convert_to_json() {
        let mut map = HashMap::new();
        map.insert(
            "R3R-A-9-2-L5".to_string(),
            DateTime::from_millis(1571985978429),
        );
        map.insert(
            "R3T-A-5-3-K6".to_string(),
            DateTime::from_millis(1571984742668),
        );
        map.insert(
            "R3R-A-10-3-L7".to_string(),
            DateTime::from_millis(1571985444876),
        );

        let src = Bacterium {
            id: ObjectId::from_str("5db131829181e500010b93d6").unwrap(),
            has_genome: false,
            creation_time: DateTime::from_millis(1571893634109),
            modified_time: DateTime::from_millis(1690439186944),
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