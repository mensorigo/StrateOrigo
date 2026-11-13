//! `quality_intervals` 区间表写出（docs/06 §5.1）。
//!
//! 与行级 `flags` 列分开落盘：区间级异常（缺口 C2、断流 C8、维护 C10
//! 等持续一段时间的异常）单表存 `{out_dir}/quality_intervals/{venue}/{date}.parquet`，
//! 供质量评级与回测时跳过缺口区间使用。

use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Int64Array, StringArray, UInt32Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;

use crate::clean::report::QualityInterval;
use crate::error::DataError;

/// 落一个交易日的 `quality_intervals`（无区间时也建表，作为"当天无异常"的凭证）。
pub fn write_intervals(
    out_dir: &Path,
    venue: &str,
    date: &str,
    intervals: &[QualityInterval],
) -> Result<usize, DataError> {
    let dir = out_dir.join("quality_intervals").join(venue);
    std::fs::create_dir_all(&dir).map_err(|e| DataError::Parquet(format!("建目录失败：{e}")))?;
    let path = dir.join(format!("{date}.parquet"));
    let file =
        std::fs::File::create(&path).map_err(|e| DataError::Parquet(format!("创建失败：{e}")))?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("channel", DataType::Utf8, false),
        Field::new("ts_start", DataType::Int64, false),
        Field::new("ts_end", DataType::Int64, false),
        Field::new("flags", DataType::UInt32, false),
    ]));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from_iter_values(
            intervals.iter().map(|i| i.symbol.as_str()),
        )),
        Arc::new(StringArray::from_iter_values(
            intervals.iter().map(|i| i.channel.as_str()),
        )),
        Arc::new(Int64Array::from_iter_values(
            intervals.iter().map(|i| i.ts_start),
        )),
        Arc::new(Int64Array::from_iter_values(
            intervals.iter().map(|i| i.ts_end),
        )),
        Arc::new(UInt32Array::from_iter_values(
            intervals.iter().map(|i| i.flags),
        )),
    ];
    // 零行时显式给空 batch（RecordBatch 需要列数组长度一致；空列也一致）。
    let batch = RecordBatch::try_new(schema.clone(), cols)
        .map_err(|e| DataError::Parquet(format!("quality_intervals batch：{e}")))?;
    let props = parquet::file::properties::WriterProperties::builder()
        .set_compression(Compression::ZSTD(Default::default()))
        .build();
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    writer
        .write(&batch)
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    writer
        .close()
        .map_err(|e| DataError::Parquet(e.to_string()))?;
    Ok(intervals.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::RecordBatchReader;

    /// 空区间写出空表（读回 0 行）；有区间时行数与内容一致。
    #[test]
    fn intervals_roundtrip() {
        let dir = std::env::temp_dir().join("so-data-intervals");
        let _ = std::fs::remove_dir_all(&dir);
        // 空
        let n = write_intervals(&dir, "gate", "2026-11-12", &[]).unwrap();
        assert_eq!(n, 0);
        // 有内容
        let items = vec![
            QualityInterval {
                venue: "gate".into(),
                symbol: "BTC_USDT".into(),
                channel: "futures.order_book_update".into(),
                ts_start: 100,
                ts_end: 200,
                flags: 0b10,
            },
            QualityInterval {
                venue: "gate".into(),
                symbol: "ETH_USDT".into(),
                channel: "futures.book_ticker".into(),
                ts_start: 300,
                ts_end: 400,
                flags: 0b1000_0000,
            },
        ];
        let n = write_intervals(&dir, "gate", "2026-11-12", &items).unwrap();
        assert_eq!(n, 2);
        let path = dir
            .join("quality_intervals")
            .join("gate")
            .join("2026-11-12.parquet");
        assert!(path.exists());

        let file = std::fs::File::open(&path).unwrap();
        let reader = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let schema = reader.schema();
        let batches: Vec<_> = reader.into_iter().map(|b| b.unwrap()).collect();
        assert_eq!(
            schema.field_with_name("ts_start").unwrap().name(),
            "ts_start"
        );
        let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(rows, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
