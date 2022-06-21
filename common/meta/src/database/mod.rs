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

use std::fmt::{Debug, Formatter};

mod database;

pub struct KeyValue {
    pub key: String,
    pub value: Option<String>,
}

impl Debug for KeyValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyValue")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}

impl PartialEq for KeyValue {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key) && self.value == other.value
    }
}

impl Clone for KeyValue {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}
