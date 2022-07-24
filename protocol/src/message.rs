//! BNF for protocol
//! Message ::= (Prefix Space)? Command CRLF
//! Prefix ::= ':' Nickname ; Can be expanded in the future
//! Command ::= Letter+ Params*
//! Params ::= (Space Middle)* (Space ':' Trailing)?
//! Middle ::= NoColonCRLFSpace (':' | NoColonCRLFSpace)*
//! Trailing ::= ( ':' | Space | NoColonCRLFSpace )*
//! NoColonCRLFSpace ::= #x00-#x09 | #x0B-#x0C | #x0E-#x1F | #x21-#x39 | #x3B-#xFF /* No Colon, CR, LF, or Space */
//! CRLF ::= #x0D #x0A
//! Nickname ::= ascii_alphabetical
use crate::command::{parse_command, Command};
use nom::{
    character::complete::{alpha1, char},
    combinator::{map, opt},
    sequence::{pair, preceded, terminated},
    IResult,
};

/// Message ::= (Prefix Space)? Command CRLF
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub prefix: Option<Prefix>,
    pub command: Command,
}

fn parse_message(input: &str) -> IResult<&str, Message> {
    map(
        pair(opt(terminated(parse_prefix, char(' '))), parse_command),
        |(prefix, command)| Message { prefix, command },
    )(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prefix {
    nick: String,
}

/// Prefix ::= ':' Nickname ;
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
        let input = ":olly MSG :Hi!, how's it going?";
        let expected = Message {
            prefix: Some(Prefix {
                nick: "olly".to_owned(),
            }),
            command: Command::Message("Hi!, how's it going?".to_owned()),
        };

        let result = parse_message(input);
        assert_eq!(Ok(("", expected)), result);
    }
}
