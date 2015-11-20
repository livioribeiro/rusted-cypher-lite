use std::io::Read;
use rustc_serialize::Decodable;
use rustc_serialize::json::{self, DecodeResult, DecoderError, ParserError};

pub fn decode_from_reader<T: Decodable, R: Read>(reader: &mut R) -> DecodeResult<T> {
    let mut buf = String::new();
    match reader.read_to_string(&mut buf) {
        Ok(_) => {},
        Err(e) => return Err(DecoderError::ParseError(ParserError::IoError(e))),
    }

    json::decode(&buf)
}
