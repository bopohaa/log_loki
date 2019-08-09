#[macro_use]
extern crate lazy_static;
extern crate fnv;
extern crate chrono;
extern crate serde;
extern crate serde_json;
mod models;
mod log;
mod loki;

pub use models::{LogMetricConf};
pub use log::Log;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loki::{LokiStream, LokiModel};

    #[test]
    fn it_works()
    {
        let metric1 = {
            Log::get(LogMetricConf::with_label_names(&["one","two"]).and_capacity(2))
                .lock()
                .unwrap()
                .get(&["1","2"])
                .clone()
        };
        {
            let mut m = metric1.lock().unwrap();
            m.push("Message1-1".to_string());
            m.push_lazy(||"Message1-2".to_string());
            m.push_lazy(||"Message1-3".to_string());
        }
        let metric2 = {
            Log::get(LogMetricConf::with_label_names(&["three"]))
                .lock()
                .unwrap()
                .get(&["3"])
                .clone()
        };
        {
            let mut m = metric2.lock().unwrap();
            m.push("Message3-1".to_string());
            m.push("Message3-2".to_string());
            m.push("Message3-3".to_string());
        }
        let metric3 = {
            Log::get(LogMetricConf::with_label_names(&["three"]))
                .lock()
                .unwrap()
                .get(&["4"])
                .clone()
        };
        {
            let mut m = metric3.lock().unwrap();
            m.push("Message4-1".to_string());
            m.push("Message4-2".to_string());
            m.push("Message4-3".to_string());
        }
        let metric3 = {
            Log::get(LogMetricConf::with_label_names(&["three"]))
                .lock()
                .unwrap()
                .get(&["4"])
                .clone()
        };
        {
            let mut m = metric3.lock().unwrap();
            m.push("Message4-4".to_string());
        }


        let data:LokiModel = Log::map(|e|LokiStream::from(e)).into();
        let json = serde_json::to_string(&data).unwrap();
        println!("{}",json);
    }
}
