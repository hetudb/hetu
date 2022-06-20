<!---
  Copyright 2021 HetuDB.
  
  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at
  
      http://www.apache.org/licenses/LICENSE-2.0
  
  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License
-->

# prost-helpers

This crate provides a derive macro `prost_helpers::AnyPB` for every prost generated message. It automatically generates
getter methods:

```rust
#[derive(prost_helpers::AnyPB, Clone, PartialEq, ::prost::Message)]
struct FooMessage {
  #[prost(message, optional, tag = "1")]
  pub field: ::core::option::Option<Field>,
  #[prost(enumeration = "foo_message::EnumFieldType", tag = "1")]
  pub enum_field: i32,
}

impl FooMessage {
  pub fn get_field(&self) -> Field {
    self.field.unwrap()
  }

  // // Later we will return Result instead of unwrap.
  // pub fn get_field(&self) -> Result<Field> {
  //  self.field.ok_or_else(InvalidArgument("field is missing in FooMessage"))
  // }

  pub fn get_enum_field(&self) -> foo_message::EnumFieldType {
    self.enum_field
  }
}
```

A distinction of prost is that it represents every optional message field using `Option`, which will lead to many
Some/None matching boilerplate code. This problem is planned to be addressed by returning `Result` specifically for the
getter of `Option` types. Therefore, the presence of fields will be validated before actually used.
