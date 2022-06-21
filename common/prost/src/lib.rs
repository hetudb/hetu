// Copyright 2021 Hetudb.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::all)]
#![allow(rustdoc::bare_urls)]

#[rustfmt::skip]
#[cfg_attr(madsim, path = "sim/common.rs")]
pub mod common;
#[rustfmt::skip]
#[cfg_attr(madsim, path = "sim/meta.rs")]
pub mod meta;
#[rustfmt::skip]
#[cfg_attr(madsim, path = "sim/photon.rs")]
pub mod photon;
#[rustfmt::skip]
#[cfg_attr(madsim, path = "sim/schema.rs")]
pub mod schema;
#[rustfmt::skip]
#[cfg_attr(madsim, path = "sim/security.rs")]
pub mod security;

#[rustfmt::skip]
#[path = "common.serde.rs"]
pub mod common_serde;
#[rustfmt::skip]
#[path = "meta.serde.rs"]
pub mod meta_serde;
#[rustfmt::skip]
#[path = "photon.serde.rs"]
pub mod photon_serde;
#[rustfmt::skip]
#[path = "schema.serde.rs"]
pub mod schema_serde;
#[rustfmt::skip]
#[path = "security.serde.rs"]
pub mod security_serde;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProstFieldNotFound(pub &'static str);

#[cfg(test)]
mod tests {
    use crate::common::KeyValue;
    use crate::meta::DatabaseInfo;
    use crate::schema::schema_proto::{ColumnKeyProto, ColumnKeyTypeProto};

    #[test]
    fn test_getter() {
        let metadata = vec![
            KeyValue {
                key: "k1".to_string(),
                value: Option::from("v1".to_string()),
            },
            KeyValue {
                key: "k2".to_string(),
                value: Option::from("v2".to_string()),
            },
        ];

        let database_id: DatabaseInfo = DatabaseInfo {
            tenant_name: String::from("hetu"),
            database_name: String::from("system"),
            creation_time: Option::from(0),
            modification_time: Option::from(0),
            metadata,
            quota_in_bytes: Option::from(10000),
            quota_in_table: Option::from(10000),
            used_namespace_in_bytes: Option::from(10000),
            object_id: Option::from(1),
            update_id: Option::from(1),
        };
        assert_eq!("system", database_id.get_database_name());
    }

    #[test]
    fn test_enum_getter() {
        let mut column_key_type: ColumnKeyProto = ColumnKeyProto::default();
        column_key_type.column_key_type = ColumnKeyTypeProto::PrimaryKey as i32;
        assert_eq!(
            ColumnKeyTypeProto::PrimaryKey,
            column_key_type.get_column_key_type().unwrap()
        );
    }

    #[test]
    fn test_primitive_getter() {
        let new_id = DatabaseInfo {
            tenant_name: "hetu".to_string(),
            database_name: "system".to_string(),
            creation_time: None,
            modification_time: None,
            metadata: vec![],
            quota_in_bytes: None,
            quota_in_table: None,
            used_namespace_in_bytes: None,
            object_id: Option::from(1),
            update_id: None,
        };
        assert_eq!(new_id.object_id.unwrap(), 1);
    }
}
