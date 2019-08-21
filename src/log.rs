use std::sync::{Mutex,Arc};
use std::collections::{HashMap};
use std::collections::vec_deque::{VecDeque};
use std::vec::Vec;
use std::iter::Iterator;
use std::hash::Hasher;
use fnv::FnvHasher;

use super::models::{LogMessage, LogMetricConf};
use std::borrow::BorrowMut;

pub struct LogMetric {
    _labels: Vec<String>,
    _messages: VecDeque<LogMessage>,
    _config: Arc<LogMetricConf>,
    _capacity: usize,
}

impl LogMetric {
    pub fn with_labels(config:Arc<LogMetricConf>, labels:&[&str])->Self{
        let default_capacity = config.get_default_capacity();
        LogMetric {
            _labels:labels.iter().map(|s| (*s).to_owned()).collect(),
            _messages: match default_capacity { 0 => VecDeque::new(),v@_ => VecDeque::with_capacity(v)},
            _capacity: default_capacity,
            _config: config,
        }
    }

    pub fn set_capacity(&mut self, capacity:usize){
        self._capacity = capacity;
    }

    pub fn config(&self)->&Arc<LogMetricConf>{
        &self._config
    }

    pub fn labels(&self)->&Vec<String>{
        &self._labels
    }

    pub fn len(&self)->usize{
        self._messages.len()
    }

    pub fn push(&mut self, message:String)->Option<()>{
        self.can_push()?;
        self._messages.push_back(message.into());
        Some(())
    }

    pub fn push_lazy<F,Ft>(&mut self, get_msg:F)->Option<()>
        where Ft:Into<LogMessage>, F:FnOnce()->Ft
    {
        self.can_push()?;
        self._messages.push_back(get_msg().into());
        Some(())
    }

    pub fn pop(&mut self)->Option<LogMessage>{
        self._messages.pop_front()
    }

    pub fn can_push(&self)->Option<()>{
        if self._capacity > 0 && self._messages.len() == self._capacity{
            return None;
        }
        Some(())
    }
}

pub struct LogContainer {
    _config: Arc<LogMetricConf>,
    _metrics: HashMap<u64, Arc<Mutex<LogMetric>>>
}

impl LogContainer {
    pub fn with_config(config: LogMetricConf)->Self{
        LogContainer {
            _config: Arc::new(config),
            _metrics: HashMap::new(),
        }
    }

    pub fn get(&mut self, labels:&[& str])->Arc<Mutex<LogMetric>>{
        assert_eq!(self._config.get_label_names().len(), labels.len());

        let key = LogContainer::get_key(labels);
        let conf= self._config.clone();
        self._metrics
            .entry(key)
            .or_insert_with(||Arc::new(Mutex::new(LogMetric::with_labels(conf, labels))))
            .clone()
    }

    pub fn map<F, R>(&self, mut map:F)->Vec<R>
        where F:FnMut(&mut LogMetric)->R {
        self._metrics.iter().map(|(_,e)|map(e.lock().unwrap().borrow_mut())).collect()
    }

    pub fn values(&self)->std::collections::hash_map::Values<'_, u64, Arc<Mutex<LogMetric>>>{
        self._metrics.values()
    }

    pub fn set_capacity_for_all(&self, capacity:usize){
        for (_,v) in self._metrics.iter(){
            v.lock().unwrap().set_capacity(capacity);
        }
    }

    fn get_key(labels: &[&str]) -> u64 {
        let mut h = FnvHasher::default();
        for val in labels {
            h.write(val.as_bytes());
        }

        h.finish()
    }

}

pub struct Log;
lazy_static!{
    static ref CONTAINERS: Mutex<HashMap<u64, Arc<Mutex<LogContainer>>>> = Mutex::new(HashMap::new());
}

#[allow(dead_code)]
impl Log {
    pub fn create(config:LogMetricConf)->Option<Arc<Mutex<LogContainer>>>{
        use std::collections::hash_map::Entry::*;
        match CONTAINERS.lock().unwrap().entry(config.get_key()) {
            Vacant(v)=>Some(v.insert(Arc::new(Mutex::new(LogContainer::with_config(config)))).clone()),
            Occupied(_)=>None
        }
    }

    pub fn get(config:LogMetricConf)->Arc<Mutex<LogContainer>>{
        CONTAINERS.lock().unwrap()
            .entry(config.get_key())
            .or_insert_with(||Arc::new(Mutex::new(LogContainer::with_config(config))))
            .clone()
    }

    pub fn map<F, R>(mut map:F)->Vec<R>
        where F:FnMut(&mut LogMetric)->R
    {
        CONTAINERS.lock().unwrap()
            .iter()
            .flat_map(|(_, container)| container.lock().unwrap().map(|e| map(e)))
            .collect()
    }
}
