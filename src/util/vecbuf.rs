use std::vec::Vec;

pub struct VecBuf<'a> {
    _data: &'a mut Vec<u8>,
    _offset : usize,
}

impl<'a> From<&'a mut Vec<u8>> for VecBuf<'a>{
    fn from(buf: &'a mut Vec<u8>)->Self{
        VecBuf { _data: buf, _offset:0 }
    }
}

#[allow(dead_code)]
impl<'a> VecBuf<'a> {
    pub fn offset(&self)->usize{
        self._offset
    }
    pub fn len(&self)->usize{
        self._data.len() - self._offset
    }
    pub fn set_offset(&mut self, offset:usize){
        self._offset = offset;
    }
    pub fn clear(&mut self){
        self._offset = 0;
        self._data.clear();
    }

}

impl<'a> std::io::Write for VecBuf<'a>{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>{
        let size = buf.len();
        self._data.extend_from_slice(buf);
        Ok(size)
    }
    fn flush(&mut self) -> std::io::Result<()>{
        Ok(())
    }
}

impl<'a> std::io::Read for VecBuf<'a>{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>{
        let needed = buf.len();
        let mut current = self.len();
        if current==0 {
            return Ok(0);
        }
        if needed<current {
            current = needed
        }
        let src = &self._data.as_slice()[self._offset..self._offset+current];
        buf.copy_from_slice(src);
        self._offset += current;

        Ok(current)
    }
}
