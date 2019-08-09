use std::time::SystemTime;
use std::hash::Hasher;
use fnv::FnvHasher;

const DEFAULT_CAPACITY:usize=1024;

pub struct LogMetricConf {
    label_names:Vec<String>,
    default_capacity: usize,
    key:u64
}
impl LogMetricConf {
    pub fn with_label_names(label_names: &[&str])->Self{
        LogMetricConf{
            label_names: label_names.iter().map(|s| (*s).to_owned()).collect(),
            default_capacity: DEFAULT_CAPACITY,
            key: LogMetricConf::create_key(label_names)
        }
    }

    pub fn and_capacity(mut self, capacity:usize)->Self{
        self.default_capacity = capacity;
        self
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

    fn create_key(labels: &[&str]) -> u64 {
        let mut h = FnvHasher::default();
        for val in labels {
            h.write(val.as_bytes());
        }

        h.finish()
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