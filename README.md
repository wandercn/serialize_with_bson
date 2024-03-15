# serialize_with_bson
bson DateTime 转json 帮助函数，转换指定时区和格式的时间格式

# Example 

```
use serialize_with_bson::{datetime_to_tz, datetime_to_tz_map, layout::DEFAULT, object_to_hex};
use bson::{doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::{collections::HashMap, str::FromStr};
 
    time_zone_and_layout!("Asia/Tokyo", DEFAULT); // 给 bson::DateTime 设定时区和时间字符串输出格式

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Bacterium {
        #[serde(rename = "_id", serialize_with = "object_to_hex")] // 启用自定义序列化函数 object_to_hex 把ObjectID转成hex字符串
        pub id: ObjectId,
        pub has_genome: bool,
        #[serde(serialize_with = "datetime_to_tz")] // 启用自定义序列化函数datetime_to_tz 把DateTime转到指定时区时间
        pub creation_time: DateTime,
        #[serde(serialize_with = "datetime_to_tz")]
        pub modified_time: DateTime,
        pub short_id: String,
        #[serde(serialize_with = "datetime_to_tz_map")] // 启用自定义序列化函数datetime_to_tz_map 把Map中的DateTime转到指定时区时间
        pub locations: HashMap<String, DateTime>,
        pub taxonomy: Document,
        pub backtrace: Vec<String>,
    }

    
fn main() {
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

```
