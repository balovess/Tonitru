use crate::internal::error::{Error, Result};

/// Encodes an unsigned 64-bit integer using a variable-length scheme (similar to LEB128).
/// Returns the encoded bytes.
pub fn encode_varint(value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut value = value;

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
    buf
}

/// Decodes an unsigned 64-bit integer from a variable-length encoded byte slice.
/// Returns the decoded value and the number of bytes read.
pub fn decode_varint(data: &[u8]) -> Result<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0;
    let mut bytes_read = 0;

    for byte in data {
        bytes_read += 1;
        let low_seven_bits = (byte & 0x7F) as u64;
        value |= low_seven_bits << shift;
        if (byte & 0x80) == 0 {
            return Ok((value, bytes_read));
        }
        shift += 7;
        if shift >= 64 {
            // Value is too large to fit in u64
            return Err(Error::CodecError("Varint value too large".to_string()));
        }
    }

    // Incomplete varint
    Err(Error::CodecError("Incomplete varint data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_varint() {
        assert_eq!(encode_varint(0), vec![0x00]);
        assert_eq!(encode_varint(1), vec![0x01]);
        assert_eq!(encode_varint(127), vec![0x7F]);
        assert_eq!(encode_varint(128), vec![0x80, 0x01]);
        assert_eq!(encode_varint(255), vec![0xFF, 0x01]);
        assert_eq!(encode_varint(300), vec![0xAC, 0x02]);
        assert_eq!(encode_varint(u64::MAX), vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]);
    }

    #[test]
    fn test_decode_varint() {
        assert_eq!(decode_varint(&[0x00]).unwrap(), (0, 1));
        assert_eq!(decode_varint(&[0x01]).unwrap(), (1, 1));
        assert_eq!(decode_varint(&[0x7F]).unwrap(), (127, 1));
        assert_eq!(decode_varint(&[0x80, 0x01]).unwrap(), (128, 2));
        assert_eq!(decode_varint(&[0xFF, 0x01]).unwrap(), (255, 2));
        assert_eq!(decode_varint(&[0xAC, 0x02]).unwrap(), (300, 2));
        assert_eq!(decode_varint(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]).unwrap(), (u64::MAX, 10));
    }

    #[test]
    fn test_decode_varint_incomplete() {
        assert!(decode_varint(&[0x80]).is_err());
        assert!(decode_varint(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).is_err());
    }

    #[test]
    fn test_decode_varint_too_large() {
        // A varint that would result in a value > u64::MAX
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert!(decode_varint(&data).is_err());
    }
}