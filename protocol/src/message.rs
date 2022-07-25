//! Message protocol.
//!
//! BNF for protocol:
//!
//! ```text
//! Message ::= (Prefix Space)? Command CRLF
//! Prefix ::= ':' Nickname /* Can be expanded in the future */
//! Command ::= Letter+ Params*
//! Params ::= (Space Middle)* (Space ':' Trailing)?
//! Middle ::= NoColonCRLFSpace (':' | NoColonCRLFSpace)*
//! Trailing ::= ( ':' | Space | NoColonCRLFSpace )*
//! NoColonCRLFSpace ::= #x00-#x09 | #x0B-#x0C | #x0E-#x1F | #x21-#x39 | #x3B-#xFF /* No Colon, CR, LF, or Space */
//! CRLF ::= #x0D #x0A
//! Nickname ::= ascii_alphabetical
//! ```
use std::fmt;
use std::str::FromStr;

use crate::command::{parse_command, Command};
use nom::{
    character::complete::{alpha1, char, crlf},
    combinator::{complete, map, opt},
    sequence::{pair, preceded, terminated},
    IResult,
};

/// A parsed message.
#[derive(Debug, Clone, PartialEq)]
pub struct LanChatMessage {
    /// Optional Prefix, when forwarding messages from one client to another the server will add
    /// a `Prefix` to show the origin of the message. Messages from client to server should not
    /// contain a prefix.
    pub prefix: Option<Prefix>,
    /// Command contained in the message, for example a [`Command::Message`] sent from a client to the server
    /// will result in a message sent to all clients connected to the server.
    pub command: Command,
}

impl fmt::Display for LanChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prefix) = &self.prefix {
            write!(f, "{} ", prefix)?;
        }

        write!(f, "{}\r\n", self.command)
    }
}

// TODO: Add more variants corresponding to the part of the message that caused the parsing error.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseMessageError {
    All,
}

impl fmt::Display for ParseMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseMessageError::*;
        match self {
            All => f.write_str("Failed to parse message"),
        }
    }
}

impl FromStr for LanChatMessage {
    type Err = ParseMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Improve error reporting
        let (_, message) = parse_message(s).map_err(|_| ParseMessageError::All)?;

        Ok(message)
    }
}

// Message ::= (Prefix Space)? Command CRLF
fn parse_message(input: &str) -> IResult<&str, LanChatMessage> {
    complete(terminated(
        map(
            pair(opt(terminated(parse_prefix, char(' '))), parse_command),
            |(prefix, command)| LanChatMessage { prefix, command },
        ),
        crlf,
    ))(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prefix {
    pub nick: String,
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.nick)
    }
}

// Prefix ::= ':' Nickname ;
fn parse_prefix(input: &str) -> IResult<&str, Prefix> {
    map(preceded(char(':'), alpha1), |nick: &str| Prefix {
        nick: nick.to_owned(),
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_prefix_works() {
        let input = ":olly";
        let expected = Prefix {
            nick: "olly".to_owned(),
        };

        let output = parse_prefix(input);
        assert_eq!(Ok(("", expected)), output);
    }

    #[test]
    fn parse_message_works() {
        let input = ":olly MSG :Hi!, how's it going?\r\n";
        let expected = LanChatMessage {
            prefix: Some(Prefix {
                nick: "olly".to_owned(),
            }),
            command: Command::Msg("Hi!, how's it going?".to_owned()),
        };

        let result = parse_message(input);
        assert_eq!(Ok(("", expected)), result);
    }

    #[test]
    fn message_from_str() {
        let input = "NICK olly\r\n";
        let expected = LanChatMessage {
            prefix: None,
            command: Command::Nick("olly".to_owned()),
        };

        let message = input.parse::<LanChatMessage>();

        assert_eq!(Ok(expected), message);
    }
}
