use std::{cell::RefCell, usize};

use crate::{Context, FileId, Files, Location};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Eof,
    Identifier,
    SlideKeyword,
    StylingKeyword,
    ElementKeyword,
    ImportKeyword,
    TemplateKeyword,
    LetKeyword,
    Number,
    SingleChar(char),
    String,
    FormatString,
    Error,
    StyleUnitLiteral,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Trivia {
    pub leading_comments: Option<Location>,
    pub trailing_comments: Option<Location>,
    pub leading_blank_line: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token {
    pub location: Location,
    pub kind: TokenKind,
    pub trivia: Trivia,
}

impl Token {
    pub fn fabricate(kind: TokenKind, mut location: Location) -> Self {
        location.length = 0;
        Token {
            location,
            kind,
            trivia: Trivia::default(),
        }
    }

    fn eof(file: FileId, start: usize, trivia: Trivia) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::Eof,
            trivia,
        }
    }

    fn identifier(file: FileId, start: usize, trivia: Trivia) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::Identifier,
            trivia,
        }
    }

    fn number(file: FileId, start: usize, trivia: Trivia) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::Number,
            trivia,
        }
    }

    fn string(file: FileId, start: usize, trivia: Trivia) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::String,
            trivia,
        }
    }

    fn format_string(file: FileId, start: usize, trivia: Trivia) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::FormatString,
            trivia,
        }
    }

    fn single_char_token(file: FileId, start: usize, char: char, trivia: Trivia) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 0,
            },
            kind: TokenKind::SingleChar(char),
            trivia,
        }
    }

    fn error(file: FileId, start: usize) -> Token {
        Token {
            location: Location {
                file,
                start,
                length: 1,
            },
            kind: TokenKind::Error,
            trivia: Trivia::default(),
        }
    }

    fn finish(&mut self, end: usize, files: &Files) {
        self.location.set_end(end);
        if self.kind == TokenKind::Identifier {
            self.kind = match self.text(files) {
                "let" => TokenKind::LetKeyword,
                "slide" => TokenKind::SlideKeyword,
                "styling" => TokenKind::StylingKeyword,
                "element" => TokenKind::ElementKeyword,
                "import" => TokenKind::ImportKeyword,
                "template" => TokenKind::TemplateKeyword,
                _ => TokenKind::Identifier,
            };
        }
    }

    pub fn text<'a, 'b: 'a>(&'a self, files: &'b Files) -> &'a str {
        &files[self.location]
    }

    pub(crate) fn combine(a: Token, b: Token, kind: TokenKind) -> Result<Token, Token> {
        let location = Location::combine(a.location, b.location);
        if a.location.end() != b.location.start {
            // TODO: This would loose data:
            // ```sld
            // 12 // Hello
            // %
            // ```
            Err(Token {
                location,
                kind: TokenKind::Error,
                trivia: a.trivia,
            })
        } else {
            Ok(Token {
                location,
                kind,
                trivia: Trivia {
                    leading_comments: a.trivia.leading_comments,
                    trailing_comments: b.trivia.trailing_comments,
                    leading_blank_line: a.trivia.leading_blank_line,
                },
            })
        }
    }
}

pub fn lex(file: crate::FileId, context: &mut crate::Context) -> Vec<Token> {
    lex_source(
        Location {
            file,
            start: 0,
            length: context.loaded_files[file].content().len(),
        },
        context,
    )
}

pub fn debug_tokens(tokens: &[Token], files: &Files) {
    for token in tokens {
        print!("Token: {:?} >{}<", token.kind, token.text(files));
        if let Some(comment) = token.trivia.leading_comments {
            print!(" // trivia-comment-before: {}", files[comment].trim());
        }
        if let Some(comment) = token.trivia.trailing_comments {
            print!(" // trivia-comment-after: {}", files[comment].trim());
        }
        println!();
    }
}

fn is_token(char: char) -> bool {
    matches!(
        char,
        ':' | ';'
            | '='
            | '('
            | ')'
            | '.'
            | ','
            | '{'
            | '}'
            | '['
            | ']'
            | '%'
            | '-'
            | '+'
            | '/'
            | '*'
            | '|'
            | '&'
    )
}

pub(crate) fn lex_source(location: Location, context: &mut Context) -> Vec<Token> {
    let Context {
        loaded_files,
        diagnostics,
        ..
    } = context;
    let file = location.file;
    let offset = location.start;
    let text_len = loaded_files[location].len();

    let mut iter = loaded_files[location]
        .char_indices()
        .chain(std::iter::once((text_len, '\0')))
        .peekable();

    #[derive(Debug, Clone, Copy)]
    enum State {
        Init,
        Identifier,
        Number,
        DecimalNumber,
        OneLineString,
        OneLineFormatString(usize),
        EscapedMultiLineString,
        LineComment,
        // Whitespace,
    }
    let mut current_token: Option<Token> = None;
    let mut current_trivia = Trivia::default();
    let mut is_comment_on_same_line_as_token = false;
    let result = RefCell::new(Vec::new());
    let mut state = State::Init;
    let finish_token = |index: usize, token: Option<Token>| {
        if let Some(mut token) = token {
            token.finish(index, &loaded_files);
            result.borrow_mut().push(token);
        }
    };
    let finish_trivia = |index: usize, trivia: &mut Trivia| {
        if let Some(comment_before) = &mut trivia.leading_comments {
            comment_before.set_end(index);
        }
    };
    let mut is_empty_line = false;
    while let Some(&(index, char)) = iter.peek() {
        let index = index + offset;
        match state {
            State::Init => match char {
                '/' => {
                    iter.next();
                    if iter.peek().is_some_and(|&(_, c)| c == '/') {
                        state = State::LineComment;
                        finish_token(index, current_token);
                        let last_token = result
                            .borrow()
                            .last()
                            .map(|t| t.location.end())
                            // HACK: This is only None, if there were no tokens
                            // yet, which means the file started with a comment.
                            // This should ensure, that the found comment will
                            // be treated as leading comment. This will fail, if
                            // the file is only made of one line, which is a
                            // comment.
                            .unwrap_or(usize::MAX);
                        let line_number_last_token = loaded_files[file].line_number(last_token);
                        let line_number_comment = loaded_files[file].line_number(index);
                        let mut result_mut = result.borrow_mut();
                        is_comment_on_same_line_as_token =
                            line_number_comment == line_number_last_token;
                        let location = if line_number_comment != line_number_last_token {
                            &mut current_trivia.leading_comments
                        } else {
                            &mut result_mut
                                .last_mut()
                                .expect("have token on same line, there must be a token then!")
                                .trivia
                                .trailing_comments
                        };
                        location
                            .get_or_insert(Location {
                                file,
                                start: index,
                                length: 0,
                            })
                            .set_end(index);
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
                    is_empty_line = false;
                }
                '\'' => {
                    finish_token(index, current_token.take());
                    finish_trivia(index, &mut current_trivia);
                    current_token = Some(Token::format_string(file, index, current_trivia));
                    current_trivia = Trivia::default();
                    state = State::OneLineFormatString(0);
                    iter.next();
                    is_empty_line = false;
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
                            current_token = Some(Token::string(file, index, current_trivia));
                            current_trivia = Trivia::default();
                            State::OneLineString
                        };
                    } else {
                        finish_token(index, current_token.take());
                        finish_trivia(index, &mut current_trivia);
                        current_token = Some(Token::string(file, index, current_trivia));
                        current_trivia = Trivia::default();
                        state = State::OneLineString;
                    }
                    iter.next();
                    is_empty_line = false;
                }
                number if number.is_ascii_digit() => {
                    state = State::Number;
                    finish_token(index, current_token.take());
                    finish_trivia(index, &mut current_trivia);
                    current_token = Some(Token::number(file, index, current_trivia));
                    current_trivia = Trivia::default();
                    iter.next();
                    is_empty_line = false;
                }
                '\0' => {
                    finish_token(index, current_token.take());
                    finish_trivia(index, &mut current_trivia);

                    finish_token(index, Some(Token::eof(file, index, current_trivia)));
                    iter.next();
                }
                whitespace if whitespace.is_ascii_whitespace() => {
                    if whitespace == '\n' {
                        if is_empty_line {
                            current_trivia.leading_blank_line = true;
                        }
                        is_empty_line = true;
                    }
                    finish_token(index, current_token.take());
                    iter.next();
                }
                alphabet if alphabet.is_ascii_alphabetic() => {
                    state = State::Identifier;
                    finish_token(index, current_token.take());
                    finish_trivia(index, &mut current_trivia);
                    current_token = Some(Token::identifier(file, index, current_trivia));
                    current_trivia = Trivia::default();
                    is_empty_line = false;
                }
                single_char_token if is_token(single_char_token) => {
                    finish_token(index, current_token.take());
                    current_token = Some(Token::single_char_token(
                        file,
                        index,
                        single_char_token,
                        current_trivia,
                    ));
                    finish_trivia(index, &mut current_trivia);
                    current_trivia = Trivia::default();
                    iter.next();
                    is_empty_line = false;
                }
                err => {
                    finish_token(index, current_token.take());
                    current_token = Some(Token::error(file, index));
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
                    is_empty_line = false;
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
            State::OneLineFormatString(open_braces) => {
                if char == '\'' {
                    if open_braces != 0 {
                        todo!("Do we handle {{{{ correctly?");
                    }
                    state = State::Init;
                } else if char == '{' {
                    state = State::OneLineFormatString(open_braces + 1)
                } else if char == '}' {
                    if open_braces == 0 {
                        todo!("Do we handle }}}} correctly?");
                    }
                    state = State::OneLineFormatString(open_braces - 1)
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
                if char == '\n' || char == '\0' {
                    if is_comment_on_same_line_as_token {
                        result
                            .borrow_mut()
                            .last_mut()
                            .expect("Should have been available")
                            .trivia
                            .trailing_comments
                            .as_mut()
                            .expect("Should have been set")
                            .set_end(index);
                    } else {
                        current_trivia
                            .leading_comments
                            .expect("Should have been set")
                            .set_end(index);
                    }
                    state = State::Init;
                }
                iter.next();
            }
        }
    }

    result.into_inner()
}
