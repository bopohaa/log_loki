use chrono::{DateTime, Utc};

use crate::models::LogMessage;
use crate::log::LogMetric;

mod scrape;

pub use scrape::LokiScrapeConfig;

const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.6f%:z";

#[derive(Debug)]
pub struct LokiModel {
    pub streams: Vec<LokiStream>
}

impl From<Vec<LokiStream>> for LokiModel{
    fn from(streams:Vec<LokiStream>)->Self{
        LokiModel{streams}
    }
}

#[derive(Debug)]
pub struct LokiStream {
    pub labels: String,
    pub entries: Vec<LokiEntry>
}

#[allow(dead_code)]
impl LokiStream {
    pub fn len(&self)->usize{
        self.entries.len()
    }
}

impl From<&mut LogMetric> for LokiStream {
    fn from(metric:&mut LogMetric)->Self{
        let mut labels = "{".to_string();
        let names = metric.config().get_label_names();
        let values = metric.labels();
        labels.push_str(names.iter().enumerate().map(|(i,e)|format!("{}=\"{}\"",e,values[i])).collect::<Vec<_>>().join(",").as_str());
        labels.push_str("}");

        let mut entries:Vec<LokiEntry> = Vec::with_capacity(metric.len());

        while let Some(v) = metric.pop(){
            entries.push(v.into());
        }

        LokiStream{
            labels,
            entries
        }
    }
}

#[derive(Debug)]
pub struct LokiEntry {
    pub ts: String,
    pub line: String
}

impl From<LogMessage> for LokiEntry {
    fn from(message:LogMessage)->Self {
        LokiEntry {
            ts: format!("{}", DateTime::<Utc>::from(message.time).format(FORMAT)),
            line: message.message,
        }
    }
}

mod protos {
    #![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
    #![allow(unused_imports)]

    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}
pub use protos::logproto;

impl<'a> From<Vec<logproto::Stream<'a>>> for logproto::PushRequest<'a> {
    fn from(streams:Vec<logproto::Stream<'a>>)->Self{
        logproto::PushRequest{streams}
    }
}

impl<'a> From<&mut LogMetric> for logproto::Stream<'a> {
    fn from(metric:&mut LogMetric)->Self {
        let mut labels = "{".to_string();
        let names = metric.config().get_label_names();
        let values = metric.labels();
        labels.push_str(names.iter().enumerate().map(|(i,e)|format!("{}=\"{}\"",e,values[i])).collect::<Vec<_>>().join(",").as_str());
        labels.push_str("}");

        let mut entries:Vec<logproto::Entry> = Vec::with_capacity(metric.len());

        while let Some(v) = metric.pop(){
            entries.push(v.into());
        }

        logproto::Stream{
            labels:  std::borrow::Cow::Owned(labels),
            entries
        }
    }
}

impl<'a> From<LogMessage> for logproto::Entry<'a> {
    fn from(message:LogMessage)->Self {
        logproto::Entry {
            ts: Some(message.time.into()),
            line: std::borrow::Cow::Owned(message.message),
        }
    }
}

impl From<std::time::SystemTime> for logproto::Timestamp {
    fn from(time: std::time::SystemTime)->Self{
        let timestamp= time.duration_since(std::time::UNIX_EPOCH).unwrap();
        logproto::Timestamp{
            seconds:timestamp.as_secs() as i64,
            nanos:timestamp.subsec_nanos() as i32
        }
    }
}