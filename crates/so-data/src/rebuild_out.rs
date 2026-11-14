//! `quality_rebuild` 重建验证表写出（docs/06 C9、§5.3 评级输入）。
//!
//! 落 `{out_root}/{n版本}/quality_rebuild/{venue}/{YYYY-MM-DD}.parquet`，
//! 每品种一行：比对数、快照/档级一致率、缺口数、比对时间窗。
//! `data check` 读回填 [`crate::clean::report::QualityInput::rebuild_match_rate`]。

use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;

use crate::bookcheck::SymbolStats;
use crate::error::DataError;

/// 落一个交易日的重建验证表（品种按名排序，确定性）。
pub fn write_rebuild(
    out_dir: &Path,
    venue: &str,
    date: &str,
    stats: &[(String, SymbolStats)],
) -> Result<usize, DataError> {
    let dir = out_dir.join("quality_rebuild").join(venue);
    std::fs::create_dir_all(&dir).map_err(|e| DataError::Parquet(format!("建目录失败：{e}")))?;
    let path = dir.join(format!("{date}.parquet"));
    let file =
        std::fs::File::create(&path).map_err(|e| DataError::Parquet(format!("创建失败：{e}")))?;

    let (schema, cols) = rebuild_columns(stats);
    let batch = RecordBatch::try_new(schema.clone(), cols)
        .map_err(|e| DataError::Parquet(format!("quality_rebuild batch：{e}")))?;
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
    Ok(stats.len())
}

/// schema 与按名排序的列。
fn rebuild_columns(stats: &[(String, SymbolStats)]) -> (Arc<Schema>, Vec<ArrayRef>) {
    let schema = Arc::new(Schema::new(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("frames", DataType::UInt64, false),
        Field::new("comparisons", DataType::UInt64, false),
        Field::new("matched", DataType::UInt64, false),
        Field::new("snapshot_rate", DataType::Float64, false),
        Field::new("level_rate", DataType::Float64, false),
        Field::new("gaps", DataType::UInt64, false),
        Field::new("stale", DataType::UInt64, false),
        Field::new("first_cmp_ts", DataType::Int64, false),
        Field::new("last_cmp_ts", DataType::Int64, false),
    ]));
    let mut sorted: Vec<&(String, SymbolStats)> = stats.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from_iter_values(
            sorted.iter().map(|(s, _)| s.as_str()),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.frames),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.comparisons),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.matched),
        )),
        Arc::new(Float64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.snapshot_rate()),
        )),
        Arc::new(Float64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.level_rate()),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.gaps),
        )),
        Arc::new(UInt64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.stale),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.first_cmp_ts),
        )),
        Arc::new(Int64Array::from_iter_values(
            sorted.iter().map(|(_, st)| st.last_cmp_ts),
        )),
    ];
    (schema, cols)
}
