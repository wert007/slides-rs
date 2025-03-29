use crate::{Context, FileId, Files, Location};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Eof,
    Identifier,
    SlideKeyword,
    StylingKeyword,
    LetKeyword,
    Number,
    SingleChar(char),
    String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token {
    pub location: Location,
    pub kind: TokenKind,
}

impl Token {
    pub fn fabricate(kind: TokenKind, mut location: Location) -> Self {
        location.length = 0;
        Token { location, kind }
    }

    fn eof(file: FileId, start: usize) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::Eof,
        }
    }

    fn identifier(file: FileId, start: usize) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::Identifier,
        }
    }

    fn number(file: FileId, start: usize) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::Number,
        }
    }

    fn string(file: FileId, start: usize) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::String,
        }
    }

    fn single_char_token(file: FileId, start: usize, char: char) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::SingleChar(char),
        }
    }

    fn finish(&mut self, end: usize, files: &Files) {
        self.location.set_end(end);
        if self.kind == TokenKind::Identifier {
            self.kind = match self.text(files) {
                "let" => TokenKind::LetKeyword,
                "slide" => TokenKind::SlideKeyword,
                "styling" => TokenKind::StylingKeyword,
                _ => TokenKind::Identifier,
            };
        }
    }

    pub fn text<'a, 'b: 'a>(&'a self, files: &'b Files) -> &'a str {
        &files[self.location]
    }
}

pub fn lex(file: crate::FileId, context: &mut crate::Context) -> Vec<Token> {
    let Context {
        loaded_files,
        diagnostics,
        ..
    } = context;
    #[derive(Debug, Clone, Copy)]
    enum State {
        Init,
        Identifier,
        Number,
        DecimalNumber,
        OneLineString,
        EscapedMultiLineString,
        LineComment,
        // Whitespace,
    }
    let mut current_token: Option<Token> = None;
    let mut result = Vec::new();
    let mut state = State::Init;
    let mut finish_token = |index: usize, token: Option<Token>| {
        if let Some(mut token) = token {
            token.finish(index, &loaded_files);
            result.push(token);
        }
    };
    let mut iter = loaded_files[file].content().char_indices().peekable();
    while let Some(&(index, char)) = iter.peek() {
        match state {
            State::Init => match char {
                '/' => {
                    iter.next();
                    if iter.peek().is_some_and(|&(_, c)| c == '/') {
                        state = State::LineComment;
                    } else {
                        diagnostics.report_unexpected_char(
                            '/',
                            Location {
                                file,
                                start: index,
                                length: 1,
                            },
                        );
                    }
                }
                '"' => {
                    if let Some(previous_token) = current_token.as_mut() {
                        previous_token.finish(index, &loaded_files);
                        let was_empty_str = previous_token.text(&loaded_files) == "\"\"";
                        let distance = index - previous_token.location.start;
                        state = if was_empty_str && distance == 2 {
                            State::EscapedMultiLineString
                        } else {
                            finish_token(index, current_token.take());
                            current_token = Some(Token::string(file, index));
                            State::OneLineString
                        };
                    } else {
                        finish_token(index, current_token.take());
                        current_token = Some(Token::string(file, index));
                        state = State::OneLineString;
                    }
                    iter.next();
                }
                number if number.is_ascii_digit() => {
                    state = State::Number;
                    finish_token(index, current_token.take());
                    current_token = Some(Token::number(file, index));
                    iter.next();
                }
                whitespace if whitespace.is_ascii_whitespace() => {
                    finish_token(index, current_token.take());
                    iter.next();
                    continue;
                }
                alphabet if alphabet.is_ascii_alphabetic() => {
                    state = State::Identifier;
                    finish_token(index, current_token.take());
                    current_token = Some(Token::identifier(file, index))
                }
                single_char_token if is_token(single_char_token) => {
                    finish_token(index, current_token.take());
                    current_token = Some(Token::single_char_token(file, index, single_char_token));
                    iter.next();
                }
                err => {
                    finish_token(index, current_token.take());
                    // debug_tokens(&result, &loaded_files);
                    diagnostics.report_unexpected_char(
                        err,
                        Location {
                            file,
                            start: index,
                            length: 1,
                        },
                    );
                    iter.next();
                }
            },
            State::Identifier => {
                if char.is_ascii_alphanumeric() || char == '_' {
                    iter.next();
                    continue;
                }
                state = State::Init;
            }
            State::Number => {
                if char.is_ascii_digit() || char == '_' {
                    iter.next();
                    continue;
                }
                if char == '.' {
                    iter.next();
                    state = State::DecimalNumber;
                } else {
                    state = State::Init;
                }
            }
            State::DecimalNumber => {
                if char.is_ascii_digit() || char == '_' {
                    iter.next();
                    continue;
                }
                state = State::Init;
            }
            State::OneLineString => {
                if char == '"' {
                    state = State::Init;
                }
                iter.next();
            }
            State::EscapedMultiLineString => {
                if char == '"' {
                    iter.next();
                    if iter.peek().is_some_and(|&(_, c)| c == '"') {
                        iter.next();
                        if iter.peek().is_some_and(|&(_, c)| c == '"') {
                            state = State::Init;
                        }
                    }
                }
                iter.next();
            }
            State::LineComment => {
                if char == '\n' {
                    state = State::Init;
                }
                iter.next();
            }
        }
    }
    let end = loaded_files[file].content().len();
    finish_token(end, current_token.take());
    finish_token(end, Some(Token::eof(file, end)));

    result
}

pub fn debug_tokens(tokens: &[Token], files: &Files) {
    for token in tokens {
        println!("Token: {:?} >{}<", token.kind, token.text(files));
    }
}

fn is_token(char: char) -> bool {
    matches!(
        char,
        ':' | ';' | '=' | '(' | ')' | '.' | ',' | '{' | '}' | '[' | ']'
    )
}
