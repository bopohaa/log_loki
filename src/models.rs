use std::time::SystemTime;
use std::hash::Hasher;
use fnv::FnvHasher;

const DEFAULT_CAPACITY:usize=1024;

pub struct LogMetricConfBuilder{
    const_labels: Vec<[String;2]>,
    label_names:Vec<String>,
    default_capacity: usize,
}

#[allow(dead_code)]
impl LogMetricConfBuilder {
    pub fn with_label_names(label_names: &[&str]) -> Self {
        LogMetricConfBuilder {
            label_names: label_names.iter().map(|s| (*s).to_owned()).collect(),
            default_capacity: DEFAULT_CAPACITY,
            const_labels: Vec::new()
        }
    }

    pub fn set_default_capacity(mut self, capacity: usize) -> Self {
        self.default_capacity = capacity;
        self
    }

    pub fn add_const_label(mut self, name: &str, value: &str) -> Self {
        self.const_labels.push([name.to_string(), value.to_string()]);
        self
    }

    pub fn build(&self)->LogMetricConf {
        LogMetricConf{
            const_labels:self.const_labels.as_slice().to_vec(),
            label_names:self.label_names.as_slice().to_vec(),
            default_capacity: self.default_capacity,
            key: self.create_key()
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

    pub fn get_key(&self)->u64{
        self.key
    }
}

pub struct LogMessage {
    pub time:SystemTime,
    pub message:String,
}
impl From<String> for LogMessage{
    fn from(msg:String)->Self{
        LogMessage{
            time: SystemTime::now(),
            message: msg
        }
    }
}