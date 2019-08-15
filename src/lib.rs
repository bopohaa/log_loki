#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate fnv;
extern crate chrono;
//extern crate minreq;
extern crate ureq;
extern crate snap;
mod errors;
mod models;
mod log;
mod loki;
mod scrape;
mod util;

pub use models::{LogMetricConf};
pub use log::Log;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loki::{LokiStream, LokiModel, LokiScrapeConfig};
    use crate::scrape::Scrape;
    use std::time::Duration;

    #[test]
    fn scrape_loki_test(){
        let scrape_conf = LokiScrapeConfig::new("http://localhost:3100/api/prom/push".to_string(), 1000, Some(3000), Some(60000), Some(30000));
        let scrape = Scrape::new();
        let log_conf = LogMetricConf::with_label_names(&["one","two"]);
        let metrics = scrape.get(log_conf);
        {
            let mut container = metrics.lock().unwrap();
            let metric = container.get(&["1", "2"]);
            {
                let mut m = metric.lock().unwrap();
                m.push("message1".to_string());
                m.push("message2".to_string());
            }
        }

        let _res = scrape.start(scrape_conf);

        std::thread::sleep(Duration::from_secs(10000));
    }

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


        let _data:LokiModel = Log::map(|e|LokiStream::from(e)).into();
    }
}
