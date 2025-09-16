
#[derive(Debug)]
pub enum RespError {
    InvalidFormat(String),
    UnexpectedEof,
    ParseIntError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for RespError {
    fn from(err: std::num::ParseIntError) -> Self {
        RespError::ParseIntError(err)
    }
}

pub fn parse_resp(input: &str) -> Result<Vec<String>, RespError>{
    let mut lines = input.lines();
    let first_line = lines.next().ok_or(RespError::UnexpectedEof)?;
    if !first_line.starts_with('*') {
        return Err(RespError::InvalidFormat("Expected *".to_string()));
    }

    let count: usize = first_line[1..].parse()?;

    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let len_line = lines.next().ok_or(RespError::UnexpectedEof)?;
        if !len_line.starts_with('$'){
            return Err(RespError::InvalidFormat("Expected $".into()));
        }

        let _len: usize = len_line[1..].parse()?;
        let value_line = lines.next().ok_or(RespError::UnexpectedEof)?;
        items.push(value_line.to_string());
    }
    return Ok(items);
}