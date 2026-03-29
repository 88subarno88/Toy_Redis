use std::io::{IoSlice, Write};
use std::net::TcpStream;

pub struct RespWriter<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> RespWriter<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self { stream }
    }

    pub fn write_simple_string(&mut self, s: &[u8]) -> std::io::Result<()> {
        write_vectored_all(self.stream, b"+", s, b"\r\n")
    }

    pub fn write_error(&mut self, msg: &[u8]) -> std::io::Result<()> {
        write_vectored_all(self.stream, b"-", msg, b"\r\n")
    }

    pub fn write_bulk_string(&mut self, data: &[u8]) -> std::io::Result<()> {
        let prefix = format!("${}\r\n", data.len());
        write_vectored_all(self.stream, prefix.as_bytes(), data, b"\r\n")
    }

    pub fn write_int(&mut self, n: i64) -> std::io::Result<()> {
        let s = format!(":{}\r\n", n);
        self.stream.write_all(s.as_bytes())
    }

    pub fn write_null(&mut self) -> std::io::Result<()> {
        self.stream.write_all(b"$-1\r\n")
    }

    pub fn write_array_header(&mut self, len: usize) -> std::io::Result<()> {
        let s = format!("*{}\r\n", len);
        self.stream.write_all(s.as_bytes())
    }
}

fn write_vectored_all(stream: &mut TcpStream, mut s1: &[u8], mut s2: &[u8], mut s3: &[u8]) -> std::io::Result<()> {
    while !s1.is_empty() || !s2.is_empty() || !s3.is_empty() {
        
  
        let mut iov = [IoSlice::new(b""), IoSlice::new(b""), IoSlice::new(b"")];
        let mut count = 0;
        
        if !s1.is_empty() { iov[count] = IoSlice::new(s1); count += 1; }
        if !s2.is_empty() { iov[count] = IoSlice::new(s2); count += 1; }
        if !s3.is_empty() { iov[count] = IoSlice::new(s3); count += 1; }
        
       
        let n = stream.write_vectored(&iov[..count])?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write whole buffer",
            ));
        }
        
      
        let mut advanced = n;
        
        if advanced > 0 && !s1.is_empty() {
            let take = advanced.min(s1.len());
            s1 = &s1[take..];
            advanced -= take;
        }
        if advanced > 0 && !s2.is_empty() {
            let take = advanced.min(s2.len());
            s2 = &s2[take..];
            advanced -= take;
        }
        if advanced > 0 && !s3.is_empty() {
            let take = advanced.min(s3.len());
            s3 = &s3[take..];
            advanced -= take;
        }
    }
    
    Ok(())
}