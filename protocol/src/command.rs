use std::fmt;

use nom::{
    bytes::complete::{take, take_while},
    character::complete::{alpha1, char},
    combinator::{map, map_res, opt, peek, verify},
    multi::many0,
    sequence::{pair, preceded},
    IResult,
};

/// Issue commands from the client to the server
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Nick(String),
    Msg(String),
    Quit,
}

impl TryFrom<(&str, Params<'_>)> for Command {
    // TODO: Error handling (avoid boxed errors)
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from((command, params): (&str, Params<'_>)) -> Result<Self, Self::Error> {
        let Params { middle, trailing } = params;
        match command {
            "NICK" => {
                if middle.len() == 1 && trailing.is_none() {
                    Ok(Command::Nick(middle[0].to_owned()))
                } else {
                    Err("Incorrect params for command: NICK".into())
                }
            }
            "MSG" => match (middle.len(), trailing) {
                (0, Some(msg)) => Ok(Command::Msg(msg.to_owned())),
                _ => Err("Incorrect params for command: MSG".into()),
            },
            "QUIT" => Ok(Command::Quit),
            other => Err(format!("Unrecognized command: {}", other).into()),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Command::*;

        match self {
            Nick(nick) => write!(f, "NICK {}", nick),
            Msg(msg) => write!(f, "MSG :{}", msg),
            Quit => f.write_str("QUIT"),
        }
    }
}

// Command ::= Letter+ Params*
pub(crate) fn parse_command(input: &str) -> IResult<&str, Command> {
    map_res(pair(alpha1, parse_params), |parsed| parsed.try_into())(input)
}

#[derive(Debug, Clone, PartialEq)]
struct Params<'a> {
    middle: Vec<&'a str>,
    trailing: Option<&'a str>,
}

/// Params ::= (Space Middle)* (' ' ':' Trailing)?
/// Middle ::= NoColonCRLFSpace (':' | NoColonCRLFSpace)*
/// Trailing ::= ( ':' | ' ' | NoColonCRLFSpace )*
fn parse_params(input: &str) -> IResult<&str, Params<'_>> {
    // Middle ::= NoColonCRLFSpace (':' | NoColonCRLFSpace)*
    let middle_param = preceded(
        verify(peek(take(1_usize)), |s: &str| s != ":"),
        take_while(|c: char| c != ' ' && c != '\n' && c != '\r'),
    );

    let trailing_param = preceded(char(':'), take_while(|c: char| c != '\n' && c != '\r'));

    map(
        pair(
            many0(preceded(char(' '), middle_param)),
            opt(preceded(char(' '), trailing_param)),
        ),
        |(middle, trailing)| Params { middle, trailing },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_params_works() {
        let input = " param1 param2 :trailing";
        let expected = Params {
            middle: vec!["param1", "param2"],
            trailing: Some("trailing"),
        };

        let result = parse_params(input);
        assert_eq!(Ok(("", expected)), result);
    }

    #[test]
    fn parse_command_message_works() {
        let input = "MSG :this is a message";
        let expected = Command::Msg("this is a message".to_owned());

        let result = parse_command(input);
        assert_eq!(Ok(("", expected)), result);
    }

    #[test]
    fn parse_command_nick_works() {
        let input = "NICK olly";
        let expected = Command::Nick("olly".to_owned());

        let result = parse_command(input);
        assert_eq!(Ok(("", expected)), result);
    }
}
