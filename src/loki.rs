use serde::{Serialize};
use chrono::{DateTime, Utc};

use crate::models::LogMessage;
use crate::log::LogMetric;

const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.6f%:z";

#[derive(Serialize, Debug)]
pub struct LokiModel {
    pub streams: Vec<LokiStream>
}

impl From<Vec<LokiStream>> for LokiModel{
    fn from(streams:Vec<LokiStream>)->Self{
        LokiModel{streams}
    }
}

#[derive(Serialize, Debug)]
pub struct LokiStream {
    pub labels: String,
    pub entries: Vec<LokiEntry>
}
impl From<&mut LogMetric> for LokiStream{
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

#[derive(Serialize, Debug)]
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