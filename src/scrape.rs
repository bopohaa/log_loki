use crate::models::{LogMetricConf};
use crate::log::{LogContainer,LogMetric,Log};
use crate::errors::*;

use std::collections::{HashMap};
use std::sync::{Mutex, Arc, atomic::AtomicBool};
use std::time::{Duration, Instant};
use std::thread::JoinHandle;

use std::sync::atomic::Ordering;
use std::cell::Cell;

type ContainersType = Arc<Mutex<HashMap<u64, Arc<Mutex<LogContainer>>>>>;

pub trait ScrapeProcess {
    fn send(&mut self, items: std::slice::Iter<'_, Arc<Mutex<LogMetric>>>)->Result<usize>;
}

pub trait ScrapeConfig {
    type ScrapeType:ScrapeProcess;
    fn get_scrape_interval(&self)->Duration;
    fn get_scrape_process(&self)->Self::ScrapeType;
}

#[allow(unused_variables)]
pub trait ScrapeEvents {
    fn on_start(&self){}
    fn on_after_scrape(&self, size:usize){}
    fn on_error(&self, err:Error){}
    fn on_end(&self){}
}

pub struct ScrapeEmptyListener;
impl ScrapeEvents for ScrapeEmptyListener{}

pub struct Scrape {
    containers:ContainersType,
    worker:Cell<Option<JoinHandle<()>>>,
    cancellation:Arc<AtomicBool>,
}

#[allow(dead_code)]
impl Scrape {
    pub fn new()->Self{
        let containers:ContainersType = Arc::new(Mutex::new(HashMap::new()));
        let cancellation = Arc::new(AtomicBool::new(false));
        Scrape{
            containers,
            worker: Cell::new(None),
            cancellation
        }
    }

    pub fn start<T>(&self, config:T)->Option<()>
        where T:'static+ScrapeConfig+Send
    {
        self.start_with_listener(config, ScrapeEmptyListener{})
    }

    pub fn start_with_listener<T,Te>(&self, config:T, events_listener:Te)->Option<()>
        where T:'static+ScrapeConfig+Send, Te:'static+ScrapeEvents+Send
    {
        let worker = self.worker.replace(None);
        if worker.is_some(){
            self.worker.replace(worker);
            return None;
        }

        let containers = self.containers.clone();
        let cancellation = self.cancellation.clone();
        let worker = std::thread::spawn(move||scrape(config, events_listener, containers, cancellation));
        self.worker.replace(Some(worker));
        Some(())
    }

    pub fn stop(&self)->Option<()> {
        let worker:JoinHandle<()> = self.worker.replace(None)?;
        self.cancellation.store(true, Ordering::Relaxed);
        worker.join().ok()
    }

    pub fn get (&self, config:LogMetricConf)->Arc<Mutex<LogContainer>>{
        self.containers.lock().unwrap()
            .entry(config.get_key())
            .or_insert_with(||Log::create(config).unwrap())
            .clone()
    }
}

impl Drop for Scrape{
    fn drop(&mut self){
        self.stop();
    }
}

#[allow(dead_code)]
fn scrape<T,Te>(config:T, event_listener:Te, containers: ContainersType, cancellation:Arc<AtomicBool>)
    where T:'static+ScrapeConfig+Send, Te:'static+ScrapeEvents+Send
{
    event_listener.on_start();
    let mut s = config.get_scrape_process();
    let interval = config.get_scrape_interval();
    let mut metrics = Vec::new();
    std::thread::sleep(interval.clone());
    let mut start = std::time::Instant::now();
    while !cancellation.load(Ordering::Relaxed) {
        for container in containers.lock().unwrap().values(){
            for metric in container.lock().unwrap().values(){
                metrics.push(metric.clone());
            }
        }

        match s.send(metrics.iter()){
            Err(err)=>event_listener.on_error(err),
            Ok(size)=>event_listener.on_after_scrape(size)
        }

        metrics.clear();
        let end = Instant::now();
        let duration = end.duration_since(start);
        start = end;

        if duration<interval {
            let duration = interval-duration;
            std::thread::sleep(duration);
            start += duration;
        }
    }
    event_listener.on_end();
}