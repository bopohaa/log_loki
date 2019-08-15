use std::sync::{Mutex, Arc};
use std::time::Duration;
use std::borrow::BorrowMut;
use quick_protobuf::message::MessageWrite;

use crate::log::LogMetric;
use crate::scrape::{ScrapeProcess, ScrapeConfig};
use crate::util::VecBuf;
use crate::errors::*;
use super::{logproto};

pub struct LokiScrapeProcess{
    loki_url:String,
    timeout_connect_ms:Option<u64>,
    timeout_write_ms:Option<u64>,
    timeout_read_ms:Option<u64>,
    buf_in: Vec<u8>,
    buf_out: Vec<u8>
}

impl LokiScrapeProcess{
    fn new(loki_url:String, timeout_connect_ms:Option<u64>, timeout_write_ms:Option<u64>, timeout_read_ms:Option<u64>)->Self{
        LokiScrapeProcess {
            loki_url,
            timeout_connect_ms,
            timeout_write_ms,
            timeout_read_ms,
            buf_in: Vec::with_capacity(65536),
            buf_out: Vec::with_capacity(65536)
        }
    }
}

impl ScrapeProcess for LokiScrapeProcess{
    fn send(&mut self, items: std::slice::Iter<'_, Arc<Mutex<LogMetric>>>)->Result<usize>{
        let mut streams = Vec::new();
        for metric in items{
            let mut g = metric.lock().unwrap();
            let s: &mut LogMetric = g.borrow_mut();
            let stream = logproto::Stream::from(s);
            if stream.entries.len()>0 {
                streams.push(stream);
            }
        }
        if streams.len()==0 {
            return Ok(0);
        }
        let data = logproto::PushRequest::from(streams);

        self.buf_in.clear();
        self.buf_out.clear();
        let mut buf_in = VecBuf::from(&mut self.buf_in);
        let mut writer = quick_protobuf::writer::Writer::new(&mut buf_in);
        data.write_message(&mut writer).map_err(|e|ErrorKind::SerializeError(e))?;

        self.buf_out.resize(snap::max_compress_len(buf_in.len()),0u8);
        let size = {
            let mut enc = snap::Encoder::new();
            enc.compress(self.buf_in.as_slice(), self.buf_out.as_mut_slice())
        }.chain_err(||"Snappy compress error")?;
        self.buf_out.truncate(size);

        let mut req = ureq::request("POST", self.loki_url.as_str());
        if let Some(timeout) = self.timeout_connect_ms{
            req.timeout_connect(timeout);
        }
        if let Some(timeout) = self.timeout_write_ms{
            req.timeout_write(timeout);
        }
        if let Some(timeout) = self.timeout_read_ms{
            req.timeout_read(timeout);
        }
        let resp = req.send_bytes(self.buf_out.as_slice());
        if resp.error(){
            let status = resp.status();
            let result = resp.into_string().unwrap_or_default();
            bail!("Send request error with status: {}, and result: '{}'", status, result);
        }
        if let Some(err) = resp.synthetic_error(){
            bail!("Send request error with status: '{}'({}), '{}'", err.status_text(), err.status(), err.body_text());
        }

        Ok(size)
    }
}

pub struct LokiScrapeConfig {
    loki_url:String,
    scrape_interval:Duration,
    timeout_connect_ms:Option<u64>,
    timeout_write_ms:Option<u64>,
    timeout_read_ms:Option<u64>,
}

#[allow(dead_code)]
impl LokiScrapeConfig {
    pub fn new(loki_url:String, scrape_interval_ms:u64, timeout_connect_ms:Option<u64>, timeout_write_ms:Option<u64>, timeout_read_ms:Option<u64>)->Self{
        LokiScrapeConfig {
            loki_url,
            scrape_interval: Duration::from_millis(scrape_interval_ms),
            timeout_connect_ms,
            timeout_write_ms,
            timeout_read_ms,
        }
    }
}

impl ScrapeConfig for LokiScrapeConfig {
    type ScrapeType = LokiScrapeProcess;

    fn get_scrape_interval(&self)->Duration {
        self.scrape_interval
    }

    fn get_scrape_process(&self)->Self::ScrapeType {
        LokiScrapeProcess::new(self.loki_url.clone(), self.timeout_connect_ms, self.timeout_write_ms, self.timeout_read_ms)
    }
}