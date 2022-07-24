use std::{cmp, fmt, io};

use bytes::{Buf, BufMut};
use tokio_util::codec::Decoder;
use tokio_util::codec::Encoder;

use crate::message::{LanChatMessage, ParseMessageError};

/// A [`Decoder`] and [`Encoder`] implementation for the LanChatProtocol based on the
/// [`LinesCodec`] codec from tokio-util
///
/// [`Decoder`]: tokio_util::codec::Decoder
/// [`Encoder`]: tokio_util::codec::Encoder
/// [`LinesCodec`]: tokio_util::codec::LinesCodec
pub struct LanChatCodec {
    // Taken from tokio-util
    // Stored index of the next index to examine for a `\n` character, used to optimise searching
    next_index: usize,

    /// The maximum length for a given message. This includes the terminating CRLF.
    max_length: usize,

    /// Are we currently discarding the remainder of a line which was over the length limit?
    is_discarding: bool,
}

impl LanChatCodec {
    /// Returns a `LanChatCodec` with a maximum length limit.
    ///
    /// If a message is over the maximum length then calls to `LanChatCodec::decode` will return a
    /// `LanChatCodecError` and subsequent calls will return `None` while discarding the remaining
    /// bytes until a CRLF is encountered, after which calls will return to normal.
    pub fn with_max_length(max_length: usize) -> LanChatCodec {
        LanChatCodec {
            next_index: 0,
            max_length,
            is_discarding: false,
        }
    }
}

impl Decoder for LanChatCodec {
    type Item = LanChatMessage;

    type Error = LanChatCodecError;

    fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            // Determine how far into the buffer we will search for a CRLF.
            // We use a saturating add incase `max_length` is `usize::MAX`
            let read_to = cmp::min(self.max_length.saturating_add(1), buf.len());

            // TODO: Maybe use iter_tools tuple windows to avoid need for bounds check or unsafe?
            let msg_end_offset = buf[self.next_index..read_to]
                .windows(2)
                .position(|slice| slice[0] == b'\r' && slice[1] == b'\n')
                .map(|pos| pos + 2); // Add 2 to the offset, if buf started with CRLF then pos
                                     // would be 0, but we want to take up to 2

            match (self.is_discarding, msg_end_offset) {
                (true, Some(msg_end_offset)) => {
                    // If we found a CRLF, discard the rest of the message.
                    // On the next iteration we will try to read normally.
                    buf.advance(msg_end_offset + self.next_index);
                    self.is_discarding = false;
                    self.next_index = 0;
                }
                (true, None) => {
                    // Otherwise we didn't find a CRLF, so we'll continue to discard the rest of
                    // the message. On the next iteration we will continue to discard the rest of
                    // the buffer unless we find a CRLF.
                    buf.advance(read_to);
                    self.next_index = 0;
                    if buf.is_empty() {
                        return Ok(None);
                    }
                }
                (false, Some(msg_end_offset)) => {
                    // Found a possible message. Try to parse and return the message.
                    let msg_end = msg_end_offset + self.next_index;
                    self.next_index = 0;
                    let msg = buf.split_to(msg_end);
                    let msg = std::str::from_utf8(&msg[..]).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "Unable to decode input as UTF8")
                    })?;
                    let msg: LanChatMessage = msg.parse()?;
                    return Ok(Some(msg));
                }
                (false, None) if buf.len() > self.max_length => {
                    // Reached max length without finding the end of the message, therefore we
                    // return an error and start discarding the message on the next call.
                    self.is_discarding = true;
                    return Err(LanChatCodecError::MaxLengthExceeded);
                }
                (false, None) => {
                    // Didn't find a full message so we set the position to resume searching for a
                    // CRLF on the next call the next call and return None.
                    self.next_index = read_to;
                    return Ok(None);
                }
            }
        }
    }
}

impl<T> Encoder<T> for LanChatMessage
where
    T: AsRef<str>,
{
    type Error = LanChatCodecError;

    fn encode(&mut self, msg: T, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let msg = msg.as_ref();
        // msg is assumed to end with CRLF
        dst.reserve(msg.len());
        dst.put(msg.as_bytes());
        Ok(())
    }
}

#[derive(Debug)]
pub enum LanChatCodecError {
    LfWithoutCr,
    MaxLengthExceeded,
    Io(io::Error),
    ParseError(ParseMessageError),
}

impl fmt::Display for LanChatCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LanChatCodecError::*;
        match self {
            // TODO: Improve error description
            LfWithoutCr => f.write_str("Message must be terminated with CRLF and not contain a LF"),
            MaxLengthExceeded => f.write_str("Maximum message length exceeded"),
            Io(e) => write!(f, "{}", e),
            ParseError(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for LanChatCodecError {
    fn from(e: io::Error) -> LanChatCodecError {
        LanChatCodecError::Io(e)
    }
}

impl From<ParseMessageError> for LanChatCodecError {
    fn from(e: ParseMessageError) -> LanChatCodecError {
        LanChatCodecError::ParseError(e)
    }
}

impl std::error::Error for LanChatCodecError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::message::Prefix;
    use bytes::BytesMut;

    #[test]
    fn lanchat_codec_happy_path() {
        let mut codec = LanChatCodec::with_max_length(100);
        let buf = &mut BytesMut::new();
        buf.reserve(100);
        buf.put_slice(b"NICK olly\r\nMSG :Hi!\r\n");

        let expected = LanChatMessage {
            prefix: None,
            command: Command::Nick("olly".to_owned()),
        };

        assert_eq!(expected, codec.decode(buf).unwrap().unwrap());

        let expected = LanChatMessage {
            prefix: None,
            command: Command::Message("Hi!".to_owned()),
        };

        assert_eq!(expected, codec.decode(buf).unwrap().unwrap());
        assert_eq!(None, codec.decode(buf).unwrap());

        buf.put_slice(b":olly MSG ");
        assert_eq!(None, codec.decode(buf).unwrap());

        buf.put_slice(b":hello???\r\n");
        let expected = LanChatMessage {
            prefix: Some(Prefix {
                nick: "olly".to_owned(),
            }),
            command: Command::Message("hello???".to_owned()),
        };
        assert_eq!(expected, codec.decode(buf).unwrap().unwrap());
    }

    #[test]
    fn lanchat_codec_unhappy_path() {
        const MAX_LENGTH: usize = 20;
        let mut codec = LanChatCodec::with_max_length(MAX_LENGTH);
        let buf = &mut BytesMut::new();

        buf.reserve(100);

        // Errors when the LanChatMessage is too long
        buf.put_slice(b"MSG :This message is too long");
        assert!(codec.decode(buf).is_err());

        // Continues to return None until it has recovered
        assert!(codec.decode(buf).unwrap().is_none());

        // Recovers once it encounters a CRLF
        buf.put_slice(b"\r\nMSG :ok!\r\n");
        let expected = LanChatMessage {
            prefix: None,
            command: Command::Message("ok!".to_owned()),
        };
        assert_eq!(expected, codec.decode(buf).unwrap().unwrap());

        // Errors on invalid LanChatMessage
        buf.put_slice(b"INVALID\r\n");
        assert!(codec.decode(buf).is_err());

        // Recovers after above error
        buf.put_slice(b"MSG :valid!\r\n");
        let expected = LanChatMessage {
            prefix: None,
            command: Command::Message("valid!".to_owned()),
        };
        assert_eq!(expected, codec.decode(buf).unwrap().unwrap());
    }
}
