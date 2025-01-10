#![deny(missing_docs)]

use std::io::{self, Read, Write};

use thiserror::Error;

/// The maximum values for a one-byte Graal-encoded integer.
pub const GUINT8_MAX: u64 = 0xDF;

/// The maximum values for a two-byte Graal-encoded integer.
pub const GUINT16_MAX: u64 = 0x705F;

/// The maximum values for a three-byte Graal-encoded integer.
pub const GUINT24_MAX: u64 = 0x38305F;

/// The maximum values for a four-byte Graal-encoded integer.
pub const GUINT32_MAX: u64 = 0x1C18305F;

/// The maximum values for a five-byte Graal-encoded integer. 60332453983 is the theoretical maximum value for a 40-bit integer,
/// but the Graal encoding only supports up to 0xFFFFFFFF in practice.
pub const GUINT40_MAX: u64 = 0xFFFFFFFF;

/// A reader that reads Graal-encoded data.
pub struct GraalReader<R: Read> {
    inner: R,
}

/// A writer that writes Graal-encoded data.
pub struct GraalWriter<W: Write> {
    inner: W,
}

/// Errors that can occur when reading or writing to a GraalReader / GraalWriter.
#[derive(Debug, Error)]
pub enum GraalIoError {
    /// A null terminator was not found when reading a string.
    #[error("No null terminator found when reading a string.")]
    NoNullTerminator(),

    /// A UTF-8 conversion failed.
    #[error("UTF8 string conversion failed: {0}")]
    Utf8ConversionFailed(String),

    /// A value exceeds the maximum for a Graal-encoded integer.
    #[error(
        "Value exceeds maximum for Graal-encoded integer. Value was {0}, but cannot exceed {1}."
    )]
    ValueExceedsMaximum(u64, u64),

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

impl<R: Read> GraalReader<R> {
    /// Creates a new GraalReader
    ///
    /// # Arguments
    /// - `inner`: The reader to wrap.
    ///
    /// # Returns
    /// - A new GraalReader with the given bytes.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let reader = GraalReader::new(Cursor::new(vec![1, 2, 3, 4]));
    /// ```
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    /// Decodes a sequence of bytes using the Graal encoding.
    ///
    /// # Arguments
    /// - `slice`: The slice of bytes to decode.
    ///
    /// # Returns
    /// - The decoded integer.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let value = GraalReader::<Cursor<Vec<u8>>>::decode_bits(&[32, 32, 32, 33]);
    /// assert_eq!(value, 1);
    /// ```
    pub fn decode_bits(slice: &[u8]) -> u64 {
        let mut value = 0;

        for (i, &byte) in slice.iter().enumerate() {
            let chunk = (byte - 32) as u64; // Remove printable offset
            let shift = 7 * (slice.len() - 1 - i);
            value += chunk << shift; // Accumulate the value
        }

        value
    }

    /// Reads a null-terminated string from the reader.
    ///
    /// # Returns
    /// - The string read from the reader.
    ///
    /// # Errors
    /// - `GraalIoError::NoNullTerminator`: If the null terminator (`0x00`) is not found.
    /// - `GraalIoError::Utf8ConversionFailed`: If the bytes cannot be converted to a UTF-8 string.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![104, 101, 108, 108, 111, 0, 119, 111, 114, 108, 100, 0]));
    /// assert_eq!(reader.read_string().unwrap(), "hello");
    /// ```
    pub fn read_string(&mut self) -> Result<String, GraalIoError> {
        let mut buffer = Vec::new();
        let mut byte = [0; 1];

        // Read bytes until a null terminator (0x00) is found
        loop {
            let bytes_read = self.inner.read(&mut byte)?;
            if bytes_read == 0 {
                return Err(GraalIoError::NoNullTerminator()); // EOF before finding null terminator
            }
            if byte[0] == 0x00 {
                break; // Null terminator found
            }
            buffer.push(byte[0]);
        }

        // Convert the buffer to a UTF-8 string
        String::from_utf8(buffer).map_err(|e| GraalIoError::Utf8ConversionFailed(e.to_string()))
    }

    /// Reads a string with a graal-encoded integer at the beginning.
    ///
    /// # Returns
    /// - The string read from the reader.
    ///
    /// # Errors
    /// - `GraalIoError::Utf8ConversionFailed`: If the bytes cannot be converted to a UTF-8 string.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    /// - `GraalIoError::ValueExceedsMaximum`: If the value exceeds the maximum for a Graal-encoded integer.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![32 + 5, 104, 101, 108, 108, 111, 32 + 5, 119, 111, 114, 108, 100]));
    /// assert_eq!(reader.read_gstring().unwrap(), "hello");
    /// assert_eq!(reader.read_gstring().unwrap(), "world");
    /// ```
    pub fn read_gstring(&mut self) -> Result<String, GraalIoError> {
        // Read the length prefix
        let length = self.read_gu8()? as usize;

        // Read the specified number of bytes for the string
        let mut chars = vec![0; length];
        self.inner.read_exact(&mut chars)?;

        // Convert bytes to a UTF-8 string
        String::from_utf8(chars).map_err(|e| GraalIoError::Utf8ConversionFailed(e.to_string()))
    }

    /// Reads an unsigned char from the reader.
    ///
    /// # Returns
    /// - The unsigned char read from the reader.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![1, 2]));
    /// assert_eq!(reader.read_u8().unwrap(), 1);
    /// assert_eq!(reader.read_u8().unwrap(), 2);
    /// ```
    pub fn read_u8(&mut self) -> Result<u8, GraalIoError> {
        let mut buffer = [0; 1];
        self.inner.read_exact(&mut buffer)?;
        Ok(buffer[0])
    }

    /// Read an unsigned short from the reader.
    ///
    /// # Returns
    /// - The unsigned short read from the reader.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![0, 1, 0, 2]));
    /// assert_eq!(reader.read_u16().unwrap(), 1);
    /// assert_eq!(reader.read_u16().unwrap(), 2);
    /// ```
    pub fn read_u16(&mut self) -> Result<u16, GraalIoError> {
        let mut buffer = [0; 2];
        self.inner.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    /// Read an unsigned int from the reader.
    ///
    /// # Returns
    /// - The unsigned int read from the reader.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![0, 0, 0, 1, 0, 0, 0, 2]));
    /// assert_eq!(reader.read_u32().unwrap(), 1);
    /// assert_eq!(reader.read_u32().unwrap(), 2);
    /// ```
    pub fn read_u32(&mut self) -> Result<u32, GraalIoError> {
        let mut buffer = [0; 4];
        self.inner.read_exact(&mut buffer)?;
        Ok(u32::from_be_bytes(buffer))
    }

    /// Reads a Graal encoded unsigned 8-bit integer from the reader.
    ///
    /// # Returns
    /// - The decoded unsigned char.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![32 + 1]));
    /// assert_eq!(reader.read_gu8().unwrap(), 1);
    /// ```
    pub fn read_gu8(&mut self) -> Result<u64, GraalIoError> {
        self.read_gu(1)
    }

    /// Reads a Graal encoded unsigned 16-bit integer from the reader.
    ///
    /// # Returns
    /// - The decoded unsigned short.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1]));
    /// assert_eq!(reader.read_gu16().unwrap(), 129);
    /// ```
    pub fn read_gu16(&mut self) -> Result<u64, GraalIoError> {
        self.read_gu(2)
    }

    /// Reads a Graal encoded unsigned 24-bit integer from the reader.
    ///
    /// # Returns
    /// - The decoded unsigned int.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1, 32 + 1]));
    /// assert_eq!(reader.read_gu24().unwrap(), 16513);
    /// ```
    pub fn read_gu24(&mut self) -> Result<u64, GraalIoError> {
        self.read_gu(3)
    }

    /// Reads a Graal encoded unsigned 32-bit integer from the reader.
    ///
    /// # Returns
    /// - The decoded unsigned int.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1, 32 + 1, 32 + 1]));
    /// assert_eq!(reader.read_gu32().unwrap(), 2113665);
    /// ```
    pub fn read_gu32(&mut self) -> Result<u64, GraalIoError> {
        self.read_gu(4)
    }

    /// Reads a Graal encoded unsigned 40-bit integer from the reader.
    ///
    /// # Returns
    /// - The decoded unsigned int.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1, 32 + 1, 32 + 1, 32 + 1]));
    /// assert_eq!(reader.read_gu40().unwrap(), 270549121);
    /// ```
    pub fn read_gu40(&mut self) -> Result<u64, GraalIoError> {
        self.read_gu(5)
    }

    /// Reads `n` bytes from the reader and decodes them as a Graal unsigned integer.
    ///
    /// # Arguments
    /// - `n`: The number of bytes to read.
    ///
    /// # Returns
    /// - The decoded Graal unsigned integer.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If an I/O error occurs
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1, 32 + 1, 32 + 1, 32 + 1]));
    /// assert_eq!(reader.read_gu(5).unwrap(), 270549121);
    /// ```
    pub fn read_gu(&mut self, n: usize) -> Result<u64, GraalIoError> {
        let mut buffer = vec![0; n];

        // Read exactly `n` bytes into the buffer
        self.inner.read_exact(&mut buffer)?;

        // Decode the buffer into a u64
        Ok(Self::decode_bits(&buffer))
    }
}

impl<W: Write> GraalWriter<W> {
    /// Creates a new GraalWriter
    ///
    /// # Arguments
    /// - `inner`: The writer to wrap.
    ///
    /// # Returns
    /// - A new GraalWriter with the given bytes.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let data = vec![0];
    /// let writer = GraalWriter::new(Cursor::new(data));
    /// ```
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Encodes a value as a sequence of bytes using the Graal encoding.
    ///
    /// # Arguments
    /// - `value`: The value to encode.
    /// - `buffer`: The buffer to write the encoded bytes to.
    /// - `byte_count`: The number of bytes to encode.
    ///
    /// # Returns
    /// - The encoded bytes.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut buffer = vec![0, 0, 0, 0];
    /// GraalWriter::<Cursor<Vec<u8>>>::encode_bits(1, &mut buffer, 4);
    /// assert_eq!(buffer, vec![32, 32, 32, 33]);
    /// ```
    pub fn encode_bits(mut value: u64, buffer: &mut Vec<u8>, byte_count: usize) {
        buffer.clear();

        for i in 0..byte_count {
            let shift = 7 * (byte_count - 1 - i);
            let chunk = (value >> shift) & 0x7F; // Extract 7 bits
            buffer.push((chunk as u8) + 32); // Add printable offset
            value -= chunk << shift; // Remove the chunk's contribution
        }
    }

    /// Writes the given bytes to the writer.
    ///
    /// # Arguments
    /// - `vec`: The bytes to write.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an I/O error.
    ///
    /// # Examples
    /// ```
    /// use std::io::Cursor;
    /// use gbf_core::graal_io::GraalWriter;
    ///
    /// let mut buffer = Cursor::new(Vec::new());
    /// let mut writer = GraalWriter::new(buffer);
    /// writer.write(&[1, 2, 3, 4]).unwrap();
    /// ```
    pub fn write(&mut self, vec: &[u8]) -> Result<(), GraalIoError> {
        self.inner.write_all(vec).map_err(GraalIoError::Io)
    }

    /// Write a string to the writer, followed by a null terminator.
    ///
    /// # Arguments
    /// - `s`: The string to write.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_string("hello").unwrap();
    /// writer.write_string("world").unwrap();
    /// ```
    pub fn write_string(&mut self, s: &str) -> Result<(), GraalIoError> {
        self.write(s.as_bytes())?;
        self.write(&[0])?;
        Ok(())
    }

    /// Writes a Graal-encoded string.
    ///
    /// # Arguments
    /// - `s`: The string to write.
    ///
    /// # Errors
    /// - `GraalIoError::ValueExceedsMaximum`: If the string is too long to be represented by a Graal encoded 8 bit integer.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_gstring("hello").unwrap();
    /// ```
    pub fn write_gstring(&mut self, s: &str) -> Result<(), GraalIoError> {
        let length = s.len();

        // Write the length prefix
        self.write_gu8(length as u64)?;

        // Write the string bytes
        self.write(s.as_bytes())?;

        Ok(())
    }

    /// Writes an unsigned 8-bit integer to the writer.
    ///
    /// # Arguments
    /// - `c`: The unsigned char to write.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_u8(1);
    /// writer.write_u8(2);
    /// writer.write_u8(3);
    /// writer.write_u8(4);
    /// ```
    pub fn write_u8(&mut self, c: u8) -> Result<(), GraalIoError> {
        // Ensure the seek location is within bounds before resizing
        self.write(&c.to_be_bytes())?;
        Ok(())
    }

    /// Write an unsigned 16-bit integer to the writer.
    ///
    /// # Arguments
    /// - `s`: The unsigned short to write.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_u16(1);
    /// ```
    pub fn write_u16(&mut self, s: u16) -> Result<(), GraalIoError> {
        let bytes = s.to_be_bytes();
        self.write(&bytes)?;
        Ok(())
    }

    /// Write an unsigned 32-bit integer to the writer.
    ///
    /// # Arguments
    /// - `i`: The unsigned int to write.
    ///
    /// # Errors
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_u32(1).unwrap();
    /// ```
    pub fn write_u32(&mut self, i: u32) -> Result<(), GraalIoError> {
        let bytes = i.to_be_bytes();
        self.write(&bytes)?;
        Ok(())
    }

    /// Writes an encoded Graal unsigned 8-bit integer.
    ///
    /// # Arguments
    /// - `v`: The value to write.
    ///
    /// # Errors
    /// - `GraalIoError::ValueExceedsMaximum`: If the value exceeds the maximum for a Graal-encoded integer.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_gu8(1).unwrap();
    /// ```
    pub fn write_gu8(&mut self, v: u64) -> Result<(), GraalIoError> {
        if v > GUINT8_MAX {
            return Err(GraalIoError::ValueExceedsMaximum(v, GUINT8_MAX));
        }

        let mut buffer = vec![0];
        Self::encode_bits(v, &mut buffer, 1);
        self.write(&buffer)?;
        Ok(())
    }

    /// Writes an encoded Graal unsigned 16-bit integer.
    ///
    /// # Arguments
    /// - `v`: The value to write.
    ///
    /// # Errors
    /// - `GraalIoError::ValueExceedsMaximum`: If the value exceeds the maximum for a Graal-encoded integer.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_gu16(1).unwrap();
    /// ```
    pub fn write_gu16(&mut self, v: u64) -> Result<(), GraalIoError> {
        if v > GUINT16_MAX {
            return Err(GraalIoError::ValueExceedsMaximum(v, GUINT16_MAX));
        }

        let mut buffer = vec![0, 0];
        Self::encode_bits(v, &mut buffer, 2);
        self.write(&buffer)?;
        Ok(())
    }

    /// Writes an encoded Graal unsigned 24-bit integer.
    ///
    /// # Arguments
    /// - `v`: The value to write.
    ///
    /// # Errors
    /// - `GraalIoError::ValueExceedsMaximum`: If the value exceeds the maximum for a Graal-encoded integer.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_gu24(1).unwrap();
    /// ```
    pub fn write_gu24(&mut self, v: u64) -> Result<(), GraalIoError> {
        if v > GUINT24_MAX {
            return Err(GraalIoError::ValueExceedsMaximum(v, GUINT24_MAX));
        }

        let mut buffer = vec![0, 0, 0];
        Self::encode_bits(v, &mut buffer, 3);
        self.write(&buffer)?;
        Ok(())
    }

    /// Writes an encoded Graal unsigned 32-bit integer.
    ///
    /// # Arguments
    /// - `v`: The value to write.
    ///
    /// # Errors
    /// - `GraalIoError::ValueExceedsMaximum`: If the value exceeds the maximum for a Graal-encoded integer.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_gu32(1).unwrap();
    /// ```
    pub fn write_gu32(&mut self, v: u64) -> Result<(), GraalIoError> {
        if v > GUINT32_MAX {
            return Err(GraalIoError::ValueExceedsMaximum(v, GUINT32_MAX));
        }

        let mut buffer = vec![0, 0, 0, 0];
        Self::encode_bits(v, &mut buffer, 4);
        self.write(&buffer)?;
        Ok(())
    }

    /// Writes an encoded Graal unsigned 40-bit integer.
    ///
    /// # Arguments
    /// - `v`: The value to write.
    ///
    /// # Errors
    /// - `GraalIoError::ValueExceedsMaximum`: If the value exceeds the maximum for a Graal-encoded integer.
    /// - `GraalIoError::Io`: If there is an underlying I/O error.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::graal_io::GraalWriter;
    /// use std::io::Cursor;
    ///
    /// let mut writer = GraalWriter::new(Cursor::new(vec![]));
    /// writer.write_gu40(1).unwrap();
    /// ```
    pub fn write_gu40(&mut self, v: u64) -> Result<(), GraalIoError> {
        if v > GUINT40_MAX {
            return Err(GraalIoError::ValueExceedsMaximum(v, GUINT40_MAX));
        }

        let mut buffer = vec![0, 0, 0, 0, 0];
        Self::encode_bits(v, &mut buffer, 5);
        self.write(&buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use io::Cursor;

    use super::*;

    const MAX_ENCODED: u8 = 0xff;
    const MIN_ENCODED: u8 = 0x20;
    const MIN_DECODED: u64 = 0;

    #[test]
    fn test_constants() {
        // Test for GUINT8
        let data = [MAX_ENCODED, MIN_ENCODED];
        let mut reader = GraalReader::new(Cursor::new(&data));
        assert_eq!(reader.read_gu8().unwrap(), GUINT8_MAX);
        assert_eq!(reader.read_gu8().unwrap(), MIN_DECODED);

        // Test for GUINT16
        let data = [MAX_ENCODED, MAX_ENCODED, MIN_ENCODED, MIN_ENCODED];
        let mut reader = GraalReader::new(Cursor::new(&data));
        assert_eq!(reader.read_gu16().unwrap(), GUINT16_MAX);
        assert_eq!(reader.read_gu16().unwrap(), MIN_DECODED);

        // Test for GUINT24
        let data = [
            MAX_ENCODED,
            MAX_ENCODED,
            MAX_ENCODED,
            MIN_ENCODED,
            MIN_ENCODED,
            MIN_ENCODED,
        ];
        let mut reader = GraalReader::new(Cursor::new(&data));
        assert_eq!(reader.read_gu24().unwrap(), GUINT24_MAX);
        assert_eq!(reader.read_gu24().unwrap(), MIN_DECODED);

        // Test for GUINT32
        let data = [
            MAX_ENCODED,
            MAX_ENCODED,
            MAX_ENCODED,
            MAX_ENCODED,
            MIN_ENCODED,
            MIN_ENCODED,
            MIN_ENCODED,
            MIN_ENCODED,
        ];
        let mut reader = GraalReader::new(Cursor::new(&data));
        assert_eq!(reader.read_gu32().unwrap(), GUINT32_MAX);
        assert_eq!(reader.read_gu32().unwrap(), MIN_DECODED);
    }

    // ===== General Operations =====

    #[test]
    fn test_new() {
        let data = vec![1, 2, 3, 4];
        let reader = GraalReader::new(&data[..]);
        assert_eq!(reader.inner, data);

        let data = vec![1, 2, 3, 4];
        let cursor = Cursor::new(data.clone());
        let writer = GraalReader::new(cursor);
        assert_eq!(writer.inner.into_inner(), data);
    }

    #[test]
    fn test_out_of_bounds() {
        // do it on a non-empty buffer
        let mut reader = GraalReader::new(Cursor::new(vec![1, 2, 3, 4]));
        assert!(reader.read_u8().is_ok());
        assert!(reader.read_u8().is_ok());
        assert!(reader.read_u8().is_ok());
        assert!(reader.read_u8().is_ok());
        assert!(reader.read_u8().is_err());

        // do it on an empty buffer
        let mut reader = GraalReader::new(Cursor::new(vec![]));
        assert!(reader.read_u8().is_err());

        // do it on a buffer with a single byte, then read a u16
        let mut reader = GraalReader::new(Cursor::new(vec![1]));
        assert!(reader.read_u16().is_err());

        // do it on a buffer with a single byte, then read a u32
        let mut reader = GraalReader::new(Cursor::new(vec![1]));
        assert!(reader.read_u32().is_err());

        // do it on a buffer with two bytes, then read a u32
        let mut reader = GraalReader::new(Cursor::new(vec![1, 2]));
        assert!(reader.read_u32().is_err());
    }

    #[test]
    fn test_decode_bits() {
        assert_eq!(
            GraalReader::<Cursor<Vec<u8>>>::decode_bits(&[32, 32, 32, 32]),
            0
        );
        assert_eq!(
            GraalReader::<Cursor<Vec<u8>>>::decode_bits(&[32, 32, 32, 33]),
            1
        );
        assert_eq!(
            GraalReader::<Cursor<Vec<u8>>>::decode_bits(&[32, 32, 33, 32]),
            128
        );
        assert_eq!(
            GraalReader::<Cursor<Vec<u8>>>::decode_bits(&[32, 33, 32, 32]),
            16384
        );
        assert_eq!(
            GraalReader::<Cursor<Vec<u8>>>::decode_bits(&[33, 32, 32, 32]),
            2097152
        );
    }

    #[test]
    fn test_encode_bits() {
        let mut reader = vec![];
        GraalWriter::<Cursor<Vec<u8>>>::encode_bits(0, &mut reader, 4);
        assert_eq!(reader, vec![32, 32, 32, 32]);

        let mut reader = vec![];
        GraalWriter::<Cursor<Vec<u8>>>::encode_bits(1, &mut reader, 4);
        assert_eq!(reader, vec![32, 32, 32, 33]);

        let mut reader = vec![];
        GraalWriter::<Cursor<Vec<u8>>>::encode_bits(128, &mut reader, 4);
        assert_eq!(reader, vec![32, 32, 33, 32]);

        let mut reader = vec![];
        GraalWriter::<Cursor<Vec<u8>>>::encode_bits(16384, &mut reader, 4);
        assert_eq!(reader, vec![32, 33, 32, 32]);

        let mut reader = vec![];
        GraalWriter::<Cursor<Vec<u8>>>::encode_bits(2097152, &mut reader, 4);
        assert_eq!(reader, vec![33, 32, 32, 32]);
    }

    // ===== Read Methods =====

    #[test]
    fn test_read_string() {
        let mut reader = GraalReader::new(Cursor::new(vec![
            104, 101, 108, 108, 111, 0, 119, 111, 114, 108, 100, 0,
        ]));
        assert_eq!(reader.read_string().unwrap(), "hello");
        assert_eq!(reader.read_string().unwrap(), "world");

        // read a string that doesn't have a null terminator
        let mut reader = GraalReader::new(Cursor::new(vec![
            104, 101, 108, 108, 111, 119, 111, 114, 108, 100,
        ]));
        assert!(reader.read_string().is_err());

        // read a string that is empty
        let mut reader = GraalReader::new(Cursor::new(vec![0]));
        assert_eq!(reader.read_string().unwrap(), "");
    }

    #[test]
    fn test_read_gstring() {
        let mut reader = GraalReader::new(Cursor::new(vec![
            32 + 5,
            104,
            101,
            108,
            108,
            111,
            32 + 5,
            119,
            111,
            114,
            108,
            100,
        ]));
        assert_eq!(reader.read_gstring().unwrap(), "hello");
        assert_eq!(reader.read_gstring().unwrap(), "world");

        // read a string that is too long
        let mut reader = GraalReader::new(Cursor::new(vec![255]));
        assert!(reader.read_gstring().is_err());
    }

    #[test]
    fn test_read_u8() {
        let mut reader = GraalReader::new(Cursor::new(vec![1, 2, 3, 4]));
        assert_eq!(reader.read_u8().unwrap(), 1);
        assert_eq!(reader.read_u8().unwrap(), 2);
        assert_eq!(reader.read_u8().unwrap(), 3);
        assert_eq!(reader.read_u8().unwrap(), 4);
    }

    #[test]
    fn test_read_u16() {
        let mut reader = GraalReader::new(Cursor::new(vec![0, 1, 0, 2]));
        assert_eq!(reader.read_u16().unwrap(), 1);
        assert_eq!(reader.read_u16().unwrap(), 2);
    }

    #[test]
    fn test_read_u32() {
        let mut reader = GraalReader::new(Cursor::new(vec![0, 0, 0, 1, 0, 0, 0, 2]));
        assert_eq!(reader.read_u32().unwrap(), 1);
        assert_eq!(reader.read_u32().unwrap(), 2);
    }

    #[test]
    fn test_read_gu8() {
        let mut reader = GraalReader::new(Cursor::new(vec![32 + 1]));
        assert_eq!(reader.read_gu8().unwrap(), 1);
    }

    #[test]
    fn test_read_gu16() {
        let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1]));
        assert_eq!(reader.read_gu16().unwrap(), 129);
    }

    #[test]
    fn test_read_gu24() {
        let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1, 32 + 1]));
        assert_eq!(reader.read_gu24().unwrap(), 16513);
    }

    #[test]
    fn test_read_gu32() {
        let mut reader = GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1, 32 + 1, 32 + 1]));
        assert_eq!(reader.read_gu32().unwrap(), 2113665);
    }

    #[test]
    fn test_read_gu40() {
        let mut reader =
            GraalReader::new(Cursor::new(vec![32 + 1, 32 + 1, 32 + 1, 32 + 1, 32 + 1]));
        assert_eq!(reader.read_gu40().unwrap(), 270549121);
    }

    // ===== Write Methods =====

    #[test]
    fn test_write_gstring() {
        // create a structure to wrap our writer
        let cursor = Cursor::new(vec![]);
        let mut writer = GraalWriter::new(cursor);
        writer.write_gstring("hello").unwrap();
        writer.write_gstring("world").unwrap();
        assert_eq!(
            writer.inner.into_inner(),
            vec![
                32 + 5,
                104,
                101,
                108,
                108,
                111,
                32 + 5,
                119,
                111,
                114,
                108,
                100
            ]
        );

        // write the max length, which should be fine
        let cursor = Cursor::new(vec![]);
        let mut writer = GraalWriter::new(cursor);
        let long_string = "a".repeat(GUINT8_MAX as usize);
        writer.write_gstring(&long_string).unwrap();
        // assert_eq!(writer.inner.into_inner(), vec![32 + GUINT8_MAX as u8] + long_string.as_bytes());

        // write a string that is too long
        let cursor = Cursor::new(vec![]);
        let mut writer = GraalWriter::new(cursor);
        let long_string = "a".repeat(GUINT8_MAX as usize + 1);
        assert!(writer.write_gstring(&long_string).is_err());
    }

    #[test]
    fn test_write_string() {
        // create new cursor
        let cursor = Cursor::new(vec![]);
        let mut writer = GraalWriter::new(cursor);
        writer.write_string("hello").unwrap();
        writer.write_string("world").unwrap();
        assert_eq!(
            writer.inner.into_inner(),
            vec![104, 101, 108, 108, 111, 0, 119, 111, 114, 108, 100, 0]
        );
    }

    #[test]
    fn test_write() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write(&[1, 2, 3, 4]).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_write_u8() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_u8(1).unwrap();
        writer.write_u8(2).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![1, 2]);
    }

    #[test]
    fn test_write_u16() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_u16(1).unwrap();
        writer.write_u16(2).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![0, 1, 0, 2]);
    }

    #[test]
    fn test_write_u32() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_u32(1).unwrap();
        writer.write_u32(2).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![0, 0, 0, 1, 0, 0, 0, 2]);
    }

    #[test]
    fn test_write_gu8() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_gu8(1).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![32 + 1]);

        // test for error case
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        assert!(writer.write_gu8(GUINT8_MAX + 1).is_err());
    }

    #[test]
    fn test_write_gu16() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_gu16(1).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![32, 32 + 1]);

        // test for error case
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        assert!(writer.write_gu16(GUINT16_MAX + 1).is_err());
    }

    #[test]
    fn test_write_gu24() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_gu24(1).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![32, 32, 32 + 1]);

        // test for error case
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        assert!(writer.write_gu24(GUINT24_MAX + 1).is_err());
    }

    #[test]
    fn test_write_gu32() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_gu32(1).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![32, 32, 32, 32 + 1]);

        // test for error case
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        assert!(writer.write_gu32(GUINT32_MAX + 1).is_err());
    }

    #[test]
    fn test_write_gu40() {
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        writer.write_gu40(1).unwrap();
        assert_eq!(writer.inner.into_inner(), vec![32, 32, 32, 32, 32 + 1]);

        // test for error case
        let mut writer = GraalWriter::new(Cursor::new(vec![]));
        assert!(writer.write_gu40(GUINT40_MAX + 1).is_err());
    }
}
