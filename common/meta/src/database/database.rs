// Copyright 2021 HetuDB.
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

use crate::database::KeyValue;
use std::borrow::Borrow;

impl From<&KeyValue> for hetu_pb::common::KeyValue {
    fn from(kv: &KeyValue) -> Self {
        hetu_pb::common::KeyValue {
            key: kv.key.clone(),
            value: kv.value.clone(),
        }
    }
}

impl From<KeyValue> for hetu_pb::common::KeyValue {
    fn from(kv: KeyValue) -> Self {
        kv.borrow().into()
    }
}

#[cfg(test)]
mod tests {
    use crate::database::KeyValue;

    #[test]
    fn key_value_to_pb() {
        let key_values: Vec<KeyValue> = vec![
            KeyValue {
                key: "k1".to_string(),
                value: Option::from("v1".to_string()),
            },
            KeyValue {
                key: "k2".to_string(),
                value: Option::from("v2".to_string()),
            },
        ];

        assert_eq!(2, key_values.len());

        let key_value = KeyValue {
            key: "k1".to_string(),
            value: Option::from("v1".to_string()),
        };
        let key_value_pb: hetu_pb::common::KeyValue = key_value.clone().into();
        assert_eq!(&key_value.key, key_value_pb.get_key());
    }
}
