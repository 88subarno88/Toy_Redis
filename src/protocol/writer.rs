use::std::io::{IoSlice,Write};
use std::net::TcpStream;
pub struct RespWriter<'a>{
    stream:& 'a mut TcpStream,
}

impl<'a>RespWriter<'a>{
    pub fn new(stream: &'a mut TcpStream)->Self{
        Self { stream }
    }

    pub fn write_simple_string(&mut self,s: &[u8])->std::io::Result<()>{
        let bufs=[
            IoSlice::new(b"+"),
            IoSlice::new(s),
            IoSlice::new(b"\r\n"),
        ];
        write_all_ina_vector(self.stream,&bufs)
    }

    pub fn write_error(&mut self,msg: &[u8])->std::io::Result<()>{
        let bufs=[
            IoSlice::new(b"-"),
            IoSlice::new(msg),
            IoSlice::new(b"\r\n"),
        ];
        write_all_ina_vector(self.stream,&bufs)
    }

    pub fn write_int(&mut self,n:i64)->std::io::Result<()>{
        let s=format!(":{}\r\n", n);
        self.stream.write_all(s.as_bytes())
    }

    pub fn write_bulk_string(&mut self,data: &[u8])->std::io::Result<()>{
          let prefix = format!("${}\r\n", data.len());
          let bufs=[
            IoSlice::new(prefix.as_bytes()),
            IoSlice::new(data),
            IoSlice::new(b"\r\n"),
          ];
          write_all_ina_vector(self.stream, &bufs)
    }

    pub fn write_null(&mut self)-> std::io::Result<()>{
        self.stream.write_all(b"$-1\r\n")
    }

    pub fn write_array_header(&mut self,len:usize)-> std::io::Result<()>{
          let s = format!("*{}\r\n", len);
          self.stream.write_all(s.as_bytes())
    }
}


fn write_all_ina_vector(stream: &mut TcpStream, bufs: &[IoSlice<'_>]) -> std::io::Result<()> {
    let mut bufs_vec = bufs.to_vec();
    let mut current_slices = &mut bufs_vec[..];

    while !current_slices.is_empty() {
        let n = stream.write_vectored(current_slices)?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write whole buffer",
            ));
        }
        
        IoSlice::advance_slices(&mut current_slices, n);
    }
    
    Ok(())
}
// fn advance_slices(bufs: &mut Vec<IoSlice>, mut n: usize){
//     bufs.retain_mut(|buf|{
//         if n==0{
//             return true;
//         }
//         if n>=buf.len(){
//             n -= buf.len();
//             false 
//         }else{
//             *buf =IoSlice::new(&buf[n..]);
//             n=0;
//             true
//         }
//     })
// }