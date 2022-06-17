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

use datafusion::arrow::array::{
    Array, BinaryArray, BooleanArray, Date32Array, Date64Array, Float16Array,
    Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
    StringArray, Time32MillisecondArray, Time32SecondArray, Time64MicrosecondArray,
    Time64NanosecondArray, TimestampMicrosecondArray, TimestampMillisecondArray,
    TimestampNanosecondArray, TimestampSecondArray, UInt16Array, UInt32Array,
    UInt64Array, UInt8Array,
};
use datafusion::arrow::datatypes::{DataType, Field, SchemaRef, TimeUnit};
use mysql_common::constants::{ColumnFlags, ColumnType};

use datafusion::arrow::record_batch::RecordBatch;
use hetu_mywire::{Column, ErrorKind, OkResponse, QueryResultWriter};

use hetu_error::{HetuError, Result};

pub struct DFQueryResultWriter<'a, W: std::io::Write> {
    inner: Option<QueryResultWriter<'a, W>>,
}

impl<'a, W: std::io::Write> DFQueryResultWriter<'a, W> {
    pub fn create(inner: QueryResultWriter<'a, W>) -> DFQueryResultWriter<'a, W> {
        DFQueryResultWriter::<'a, W> { inner: Some(inner) }
    }

    pub fn write(
        &mut self,
        query_result: Result<(Vec<RecordBatch>, String)>,
    ) -> Result<()> {
        if let Some(writer) = self.inner.take() {
            match query_result {
                Ok((blocks, extra_info)) => Self::ok(blocks, extra_info, writer)?,
                Err(error) => Self::err(&error, writer)?,
            }
        }
        Ok(())
    }

    fn ok(
        blocks: Vec<RecordBatch>,
        extra_info: String,
        dataframe_writer: QueryResultWriter<'a, W>,
    ) -> Result<()> {
        // XXX: num_columns == 0 may is error?
        let default_response = OkResponse {
            info: extra_info,
            ..Default::default()
        };

        if blocks.is_empty() || (blocks[0].num_columns() == 0) {
            dataframe_writer.completed(default_response)?;
            return Ok(());
        }

        fn convert_field_type(field: &Field) -> Result<ColumnType> {
            match field.data_type() {
                DataType::Int8 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::Int16 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::Int32 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::Int64 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::UInt8 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::UInt16 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::UInt32 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::UInt64 => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::Float16 => Ok(ColumnType::MYSQL_TYPE_FLOAT),
                DataType::Float32 => Ok(ColumnType::MYSQL_TYPE_FLOAT),
                DataType::Float64 => Ok(ColumnType::MYSQL_TYPE_FLOAT),
                DataType::Boolean => Ok(ColumnType::MYSQL_TYPE_BIT),
                DataType::Decimal(_, _) => Ok(ColumnType::MYSQL_TYPE_DECIMAL),
                DataType::Date32 | DataType::Date64 => Ok(ColumnType::MYSQL_TYPE_DATE),
                DataType::Time32(_) => Ok(ColumnType::MYSQL_TYPE_DATETIME),
                DataType::Time64(_) => Ok(ColumnType::MYSQL_TYPE_DATETIME),
                DataType::Timestamp(_, _) => Ok(ColumnType::MYSQL_TYPE_TIMESTAMP),
                DataType::Null => Ok(ColumnType::MYSQL_TYPE_NULL),
                DataType::Interval(_) => Ok(ColumnType::MYSQL_TYPE_LONG),
                DataType::Utf8 => Ok(ColumnType::MYSQL_TYPE_VARCHAR),
                DataType::LargeUtf8 => Ok(ColumnType::MYSQL_TYPE_VARCHAR),
                DataType::Binary => Ok(ColumnType::MYSQL_TYPE_BLOB),
                _ => Err(HetuError::NotImplemented(format!(
                    "column type {:?}",
                    field.data_type()
                ))),
            }
        }

        fn make_column_from_field(field: &Field) -> Result<Column> {
            convert_field_type(field).map(|column_type| Column {
                table: "".to_string(),
                column: field.name().to_string(),
                coltype: column_type,
                colflags: ColumnFlags::empty(),
            })
        }

        fn convert_schema(schema: SchemaRef) -> Result<Vec<Column>> {
            schema.fields().iter().map(make_column_from_field).collect()
        }

        let block = blocks[0].clone();
        match convert_schema(block.schema()) {
            Err(error) => Self::err(&error, dataframe_writer),
            Ok(columns) => {
                let rows = block.num_rows();
                let columns_size = block.num_columns();
                let mut row_writer = dataframe_writer.start(&columns)?;

                for block in &blocks {
                    for row_index in 0..rows {
                        for column_index in 0..columns_size {
                            if block.column(column_index).is_null(row_index) {
                                row_writer.write_col(None::<u8>)?;
                                continue;
                            }

                            let schema = block.schema();
                            let data_type = schema.fields()[column_index].data_type();
                            match data_type {
                                DataType::Boolean => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<BooleanArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index) as i8)?
                                }
                                DataType::Int8 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Int8Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Int16 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Int16Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Int32 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Int32Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Int64 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Int64Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::UInt8 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<UInt8Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::UInt16 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<UInt16Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::UInt32 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<UInt32Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::UInt64 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<UInt64Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Float16 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Float16Array>()
                                        .unwrap();
                                    row_writer
                                        .write_col(f32::from(val.value(row_index)))?
                                }
                                DataType::Float32 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Float32Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Float64 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Float64Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Utf8 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<StringArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Date32 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Date32Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Date64 => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Date64Array>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Time32(TimeUnit::Second) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Time32SecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Time32(TimeUnit::Millisecond) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Time32MillisecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Time64(TimeUnit::Microsecond) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Time64MicrosecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Time64(TimeUnit::Nanosecond) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<Time64NanosecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Timestamp(TimeUnit::Second, _) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<TimestampSecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Timestamp(TimeUnit::Millisecond, _) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<TimestampMillisecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Timestamp(TimeUnit::Microsecond, _) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<TimestampMicrosecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Timestamp(TimeUnit::Nanosecond, _) => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<TimestampNanosecondArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                DataType::Binary => {
                                    let val = block
                                        .column(column_index)
                                        .as_any()
                                        .downcast_ref::<BinaryArray>()
                                        .unwrap();
                                    row_writer.write_col(val.value(row_index))?
                                }
                                _ => {
                                    println!("Unsupported column type:{:?}", data_type);
                                    return Err(HetuError::NotImplemented(format!(
                                        "Unsupported column type:{:?}",
                                        data_type
                                    )));
                                }
                            }
                        }
                        row_writer.end_row()?;
                    }
                }
                // end
                row_writer.finish_with_info(&default_response.info)?;
                Ok(())
            }
        }
    }

    fn err(error: &HetuError, writer: QueryResultWriter<'a, W>) -> Result<()> {
        writer.error(ErrorKind::ER_UNKNOWN_ERROR, format!("{}", error).as_bytes())?;
        Ok(())
    }
}
