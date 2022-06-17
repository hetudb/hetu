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

# Hetu Cloud Service UI

## Start project from source

### Run hetu-cloud-service/hetu-query

First, run cloud-service from project:

```shell
$ cd core/cloudserv
$ RUST_LOG=info cargo run --release
...
    Finished release [optimized] target(s) in 11.92s
    Running `/path-to-project/target/release/hetu-cloud-service`
```

and run query in new terminal:

```shell
$ cd core/query
$ RUST_LOG=info cargo run --release
    Finished release [optimized] target(s) in 0.09s
    Running `/path-to-project/target/release/hetu-query`
```

### Run Client project

```shell
$ cd core/cloudsrv/ui/scheduler
$ yarn
Resolving packages...
$ yarn start
Starting the development server...
```

Now access to http://localhost:3000/
