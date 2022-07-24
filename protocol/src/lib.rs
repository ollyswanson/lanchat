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
mod command;
mod message;

