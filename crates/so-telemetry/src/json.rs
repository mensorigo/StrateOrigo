//! JSON Lines 日志格式：固定字段在前（`ts`、`level`、实例身份、`target`），事件字段在后。

use std::fmt;

use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, MakeWriter};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::registry::LookupSpan;

use crate::config::LogContext;

/// 由本库写入、事件字段不得覆盖的键。
const RESERVED_KEYS: [&str; 9] = [
    "ts", "level", "instance", "mode", "venue", "account", "strategy", "target", "msg",
];

/// 以 JSON Lines 格式输出到 `writer` 的日志层。
pub fn json_layer<S, W>(writer: W, context: &LogContext) -> impl Layer<S> + Send + Sync + use<S, W>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    W: for<'w> MakeWriter<'w> + Send + Sync + 'static,
{
    tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .event_format(JsonLines {
            static_fields: context.static_fields(),
        })
}

#[derive(Debug)]
struct JsonLines {
    static_fields: Vec<(&'static str, Value)>,
}

impl<S, N> FormatEvent<S, N> for JsonLines
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let mut record = Map::new();
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
        record.insert("ts".into(), ts.into());
        record.insert("level".into(), meta.level().as_str().into());
        for (key, value) in &self.static_fields {
            record.insert((*key).into(), value.clone());
        }
        record.insert("target".into(), meta.target().into());
        event.record(&mut FieldVisitor(&mut record));
        let line = serde_json::to_string(&record).map_err(|_| fmt::Error)?;
        writeln!(writer, "{line}")
    }
}

/// 把事件字段写入 JSON 对象；`message` 改名为 `msg`，与固定字段同名的加 `field.` 前缀。
struct FieldVisitor<'a>(&'a mut Map<String, Value>);

impl FieldVisitor<'_> {
    fn insert(&mut self, field: &Field, value: Value) {
        let name = field.name();
        let key = if name == "message" {
            "msg".to_owned()
        } else if RESERVED_KEYS.contains(&name) {
            format!("field.{name}")
        } else {
            name.to_owned()
        };
        self.0.insert(key, value);
    }
}

impl Visit for FieldVisitor<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, value.into());
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, value.into());
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, value.into());
    }
    fn record_i128(&mut self, field: &Field, value: i128) {
        self.insert(field, value.to_string().into());
    }
    fn record_u128(&mut self, field: &Field, value: u128) {
        self.insert(field, value.to_string().into());
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, value.into());
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field, value.into());
    }
    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert(field, value.to_string().into());
    }
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.insert(field, format!("{value:?}").into());
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use so_core::RunMode;
    use tracing_subscriber::layer::SubscriberExt;

    use super::*;

    #[derive(Clone, Default)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Buffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'w> MakeWriter<'w> for Buffer {
        type Writer = Buffer;
        fn make_writer(&'w self) -> Self::Writer {
            self.clone()
        }
    }

    fn live_context() -> LogContext {
        LogContext {
            instance: "live-okx-funding".into(),
            mode: Some(RunMode::Live),
            venue: Some("okx".into()),
            account: Some("okx-sub-funding".into()),
            strategy: None,
        }
    }

    fn capture(emit: impl FnOnce()) -> Vec<Map<String, Value>> {
        let buf = Buffer::default();
        let subscriber =
            tracing_subscriber::registry().with(json_layer(buf.clone(), &live_context()));
        tracing::subscriber::with_default(subscriber, emit);
        let bytes = buf.0.lock().unwrap().clone();
        String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    #[test]
    fn record_has_fixed_fields_in_order() {
        let records = capture(|| {
            tracing::warn!(
                coid = 42_u64,
                symbol = "BTC-USDT-SWAP",
                "订单簿序列缺口，重建"
            );
        });
        assert_eq!(records.len(), 1);
        let r = &records[0];
        let keys: Vec<&str> = r.keys().map(String::as_str).collect();
        // 顺序与 docs/03 §18.3 一致：固定字段 → msg → 事件字段。
        let want = [
            "ts", "level", "instance", "mode", "venue", "account", "target", "msg", "coid",
            "symbol",
        ];
        assert_eq!(keys, want);
        assert_eq!(r["level"], "WARN");
        assert_eq!(r["instance"], "live-okx-funding");
        assert_eq!(r["mode"], "live");
        assert_eq!(r["coid"], 42);
        assert_eq!(r["msg"], "订单簿序列缺口，重建");
        assert!(r["ts"].as_str().unwrap().ends_with('Z'));
    }

    #[test]
    fn event_fields_cannot_override_identity() {
        let records = capture(|| tracing::info!(mode = "paper", "伪造的模式字段"));
        assert_eq!(records[0]["mode"], "live");
        assert_eq!(records[0]["field.mode"], "paper");
    }
}
