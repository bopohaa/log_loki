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
    use crate::models::LogMetricConfBuilder;

    #[test]
    fn scrape_loki_test(){
        let scrape_conf = LokiScrapeConfig::new("http://localhost:3100/api/prom/push".to_string(), 1000, Some(3000), Some(60000), Some(30000));
        let scrape = Scrape::new();
        let log_conf = LogMetricConfBuilder::with_label_names(&["one","two"]).build();
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
        let b = LogMetricConfBuilder::with_label_names(&["three"]);
        b.set_default_capacity(10);
        b.add_const_label("one","1");
        let c = b.build();

        let metric1 = {
            Log::get(LogMetricConfBuilder::with_label_names(&["one","two"]).set_default_capacity(2).build())
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
            Log::get(LogMetricConfBuilder::with_label_names(&["three"]).build())
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
            Log::get(LogMetricConfBuilder::with_label_names(&["three"]).build())
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
            Log::get(LogMetricConfBuilder::with_label_names(&["three"]).build())
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
