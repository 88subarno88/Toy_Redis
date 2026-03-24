#[derive(Debug,PartialEq)]
pub enum RespValue<'a>{
    SimpleString(&'a[u8]),
    Error(&'a[u8]),
    Integer(i64),
    BulkString(&'a[u8]),
    Array(Vec<RespValue<'a>>),
    Null,
}
#[derive(Debug)]
pub enum ParseError{
    Incomplete,
    Invalid(&'static str),
}

pub fn parse(buf: &[u8]) -> Result<(RespValue, usize), ParseError> {
    if buf.is_empty() {
        return Err(ParseError::Incomplete);
    }
    match buf[0] {
        b'+' => parse_simple_string(buf),
        b'-' => parse_error(buf),
        b':' => parse_integer(buf),
        b'$' => parse_bulk_string(buf),
        b'*' => parse_array(buf),
        _    => Err(ParseError::Invalid("unknown type byte")),
    }
}

fn find_crlf(buf: &[u8], start: usize) -> Option<usize> {
    buf[start..].windows(2)
        .position(|w| w == b"\r\n")
        .map(|p| start + p)
}

fn parse_decimal(buf: &[u8],start:usize,end:usize)->Result<i64,ParseError>{
    let s=std::str::from_utf8(&buf[start..end])
       .map_err(|_|ParseError::Invalid("non-utf8 integer"))?;
    s.parse::<i64>()
       .map_err(|_|ParseError::Invalid("bad integer"))
}

fn parse_simple_string(buf: &[u8])->Result<(RespValue, usize),ParseError>{
    let end = find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
    Ok((RespValue::SimpleString(&buf[1..end]), end + 2))
}

fn parse_error(buf: &[u8])-> Result<(RespValue, usize),ParseError>{
    let end = find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
    Ok((RespValue::Error(&buf[1..end]), end + 2))
}

fn parse_integer(buf: &[u8])-> Result<(RespValue, usize),ParseError>{
     let end = find_crlf(buf, 1).ok_or(ParseError::Incomplete)?;
     let n=parse_decimal(buf,1,end)?;
     Ok((RespValue::Integer(n),end+2))
}

fn parse_bulk_string(buf: &[u8])-> Result<(RespValue, usize),ParseError>{
    let len_end = find_crlf(buf,1).ok_or(ParseError::Incomplete)?;
    let len=parse_decimal(buf,1,len_end)?;
    if len==-1{
        return Ok((RespValue::Null,len_end+2));
    }
    if len<0{
        return Err(ParseError::Invalid("negetive bulk length"));
    }
    let len=len as usize;
    let data_start=len_end+2;
    let data_end=data_start+len;
    if buf.len()<data_end+2{
        return Err(ParseError::Incomplete);
    }
    Ok((RespValue::BulkString(&buf[data_start..data_end]),data_end+2))
}

fn parse_array(buf: &[u8])-> Result<(RespValue, usize),ParseError>{
    let len_end = find_crlf(buf,1).ok_or(ParseError::Incomplete)?;
    let len=parse_decimal(buf,1,len_end)?;
    if len==-1{
        return Ok((RespValue::Null,len_end+2));
    }
    if len<0{
        return Err(ParseError::Invalid("negetive bulk length"));
    }
    let mut offset=len_end+2;
    let mut items =Vec::with_capacity(len as usize);
    for _ in 0..len{
        let (val,consumed) =parse(&buf[offset..])?;
        items.push(val);
        offset+=consumed;
    }
    Ok((RespValue::Array(items),offset))
}