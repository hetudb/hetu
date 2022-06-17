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

//! hetu error types

use datafusion::error::DataFusionError;
use std::error;
use std::fmt::{Display, Formatter};
use std::io;

/// Result type for operations that could result in an [HetuError]
pub type Result<T> = std::result::Result<T, HetuError>;

/// Error type for generic operations that could result in HetuError::External
pub type GenericError = Box<dyn error::Error + Send + Sync>;

/// hetu error
#[derive(Debug)]
pub enum HetuError {
    /// Error associated to I/O operations and associated traits.
    IoError(io::Error),
    /// Error returned on a branch that we know it is possible
    /// but to which we still have no implementation for.
    /// Often, these errors are tracked in our issue tracker.
    NotImplemented(String),
    /// Error returned as a consequence of an error in hetu.
    /// This error should not happen in normal usage of hetu.
    // Hetu has internal invariants that we are unable to ask the compiler to check for us.
    // This error is raised when one of those invariants is not verified during execution.
    Internal(String),
    /// Errors originating from outside hetu's core codebase.
    /// For example, a custom S3Error from the crate hetu-objectstore-s3
    External(GenericError),
    /// Error returned data fusion error message
    DataFusionError(DataFusionError),
}

impl From<io::Error> for HetuError {
    fn from(e: io::Error) -> Self {
        HetuError::IoError(e)
    }
}

impl From<GenericError> for HetuError {
    fn from(err: GenericError) -> Self {
        HetuError::External(err)
    }
}

impl From<DataFusionError> for HetuError {
    fn from(err: DataFusionError) -> Self {
        HetuError::DataFusionError(err)
    }
}

impl Display for HetuError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            HetuError::IoError(ref desc) => write!(f, "IO error: {}", desc),
            HetuError::NotImplemented(ref desc) => {
                write!(f, "This feature is not implemented: {}", desc)
            }
            HetuError::Internal(ref desc) => {
                write!(f, "Internal error: {}. This was likely caused by a bug in Hetu's \
                    code and we would welcome that you file an bug report in our issue tracker", desc)
            }
            HetuError::External(ref desc) => {
                write!(f, "External error: {}", desc)
            }
            HetuError::DataFusionError(ref desc) => {
                write!(f, "Data fusion error: {}", desc)
            }
        }
    }
}

impl error::Error for HetuError {}

#[cfg(test)]
mod test {
    use crate::error::HetuError;
    use datafusion::error::DataFusionError;

    #[test]
    fn hetu_not_impl_error() {
        let res = return_hetu_not_impl_error().unwrap_err();
        assert_eq!(
            res.to_string(),
            "This feature is not implemented: Create Table"
        );
    }

    /// Model what happens when using arrow kernels in hetu
    /// code: need to turn an ArrowError into a HetuError
    #[allow(clippy::try_err)]
    fn return_hetu_not_impl_error() -> crate::error::Result<()> {
        // Expect the '?' to work
        let _bar = Err(HetuError::NotImplemented(String::from("Create Table")))?;
        Ok(())
    }

    #[test]
    fn hetu_data_fusion_error() {
        let res = return_hetu_data_fusion_error().unwrap_err();
        assert_eq!(
            res.to_string(),
            "Data fusion error: This feature is not implemented: show tables"
        )
    }

    #[allow(clippy::try_err)]
    fn return_hetu_data_fusion_error() -> crate::error::Result<()> {
        // Expect the '?' to work
        let _bar = Err(HetuError::DataFusionError(DataFusionError::NotImplemented(
            String::from("show tables"),
        )))?;
        Ok(())
    }
}

#[macro_export]
macro_rules! internal_err {
    ($($arg:tt)*) => {
        Err(HetuError::Internal(format!($($arg)*)))
    };
}
