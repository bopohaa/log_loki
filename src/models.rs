use std::time::SystemTime;
use std::hash::Hasher;
use fnv::FnvHasher;

const DEFAULT_CAPACITY:usize=1024;

#[derive(Clone)]
pub struct LogMetricConfBuilder{
    const_labels: Vec<[String;2]>,
    label_names:Vec<String>,
    default_capacity: usize,
}

impl Default for LogMetricConfBuilder{
    fn default() -> Self{
        LogMetricConfBuilder {
            label_names: Vec::new(),
            default_capacity: DEFAULT_CAPACITY,
            const_labels: Vec::new()
        }
    }
}

#[allow(dead_code)]
impl LogMetricConfBuilder {
    pub fn new() -> Self {
        LogMetricConfBuilder::default()
    }

    pub fn set_default_capacity(mut self, capacity: usize) -> Self {
        self.default_capacity = capacity;
        self
    }

    pub fn add_label<T:Into<String>>(mut self, label_name:T)->Self{
        self.label_names.push(label_name.into());
        self
    }

    pub fn add_labels(mut self, label_names: &[&str])->Self {
        self.label_names.extend(label_names.iter().map(|e|(*e).into()));
        self
    }

    pub fn add_const_label<T:Into<String>>(mut self, name: T, value: T) -> Self {
        self.const_labels.push([name.into(), value.into()]);
        self
    }

    pub fn build(self)->LogMetricConf {
        let key = self.create_key();
        LogMetricConf{
            key,
            default_capacity: self.default_capacity,
            const_labels:self.const_labels,
            label_names:self.label_names,
        }
    }

    fn create_key(&self) -> u64 {
        let mut h = FnvHasher::default();
        for val in self.const_labels.iter() {
            h.write(val[0].as_bytes());
        }
        for val in self.label_names.iter() {
            h.write(val.as_bytes());
        }

        h.finish()
    }
}

pub struct LogMetricConf {
    const_labels: Vec<[String;2]>,
    label_names:Vec<String>,
    default_capacity: usize,
    key:u64
}
impl LogMetricConf {
    pub fn get_const_labels(&self)->&Vec<[String;2]>{
        &self.const_labels
    }

    pub fn get_label_names(&self)->&Vec<String>{
        &self.label_names
    }

    pub fn get_default_capacity(&self)->usize{
        self.default_capacity
    }

    pub fn set_default_capacity(&mut self, capacity:usize){
        self.default_capacity = capacity;
    }

    pub fn get_key(&self)->u64{
        self.key
    }
}

pub struct LogMessage {
    pub time:SystemTime,
    pub message:String,
}
impl<T: Into<String>> From<T> for LogMessage{
    fn from(msg:T)->Self{
        LogMessage{
            time: SystemTime::now(),
            message: msg.into()
        }
    }
}