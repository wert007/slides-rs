use crate::compiler::lexer::{self, debug_tokens};

use super::{
    Context, FileId,
    lexer::{Token, TokenKind},
};

pub enum SyntaxNodeKind {
    StylingStatement {
        styling_keyword: Token,
        name: Token,
        lparen: Token,
        type_: Token,
        rparen: Token,
        colon: Token,
        body: Vec<SyntaxNode>,
    },
    ExpressionStatement {
        expression: Box<SyntaxNode>,
        semicolon: Token,
    },
    VariableDeclaration {
        let_keyword: Token,
        name: Token,
        equals: Token,
        expression: Box<SyntaxNode>,
        semicolon: Token,
    },
    SlideStatement {
        slide_keyword: Token,
        name: Token,
        colon: Token,
        body: Vec<SyntaxNode>,
    },
    VariableReference {
        variable: Token,
    },
    Literal {
        literal: Token,
    },
    MemberAccess {
        base: Box<SyntaxNode>,
        period: Token,
        member: Token,
    },
    AssignmentStatement {
        expression: Box<SyntaxNode>,
        equals: Token,
        assignment: Box<SyntaxNode>,
        semicolon: Token,
    },
    FunctionCall {
        base: Box<SyntaxNode>,
        lparen: Token,
        arguments: Vec<(SyntaxNode, Option<Token>)>,
        rparen: Token,
    },
}

pub struct SyntaxNode {
    kind: SyntaxNodeKind,
}

impl SyntaxNode {
    fn styling_statement(
        styling_keyword: Token,
        name: Token,
        lparen: Token,
        type_: Token,
        rparen: Token,
        colon: Token,
        body: Vec<SyntaxNode>,
    ) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::StylingStatement {
                styling_keyword,
                name,
                lparen,
                type_,
                rparen,
                colon,
                body,
            },
        }
    }

    fn expression_statement(expression: SyntaxNode, semicolon: Token) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::ExpressionStatement {
                expression: Box::new(expression),
                semicolon,
            },
        }
    }

    fn variable_declaration(
        let_keyword: Token,
        name: Token,
        equals: Token,
        expression: SyntaxNode,
        semicolon: Token,
    ) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::VariableDeclaration {
                let_keyword,
                name,
                equals,
                expression: Box::new(expression),
                semicolon,
            },
        }
    }

    fn slide_statement(
        slide_keyword: Token,
        name: Token,
        colon: Token,
        body: Vec<SyntaxNode>,
    ) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::SlideStatement {
                slide_keyword,
                name,
                colon,
                body,
            },
        }
    }

    fn variable_reference(variable: Token) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::VariableReference { variable },
        }
    }

    fn literal(literal: Token) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::Literal { literal },
        }
    }

    fn member_access(base: SyntaxNode, period: Token, member: Token) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::MemberAccess {
                base: Box::new(base),
                period,
                member,
            },
        }
    }

    fn assignment_statement(
        expression: SyntaxNode,
        equals: Token,
        assignment: SyntaxNode,
        semicolon: Token,
    ) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::AssignmentStatement {
                expression: Box::new(expression),
                equals,
                assignment: Box::new(assignment),
                semicolon,
            },
        }
    }

    fn function_call(
        base: SyntaxNode,
        lparen: Token,
        arguments: Vec<(SyntaxNode, Option<Token>)>,
        rparen: Token,
    ) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::FunctionCall {
                base: Box::new(base),
                lparen,
                arguments,
                rparen,
            },
        }
    }
}

pub struct Ast {
    statements: Vec<SyntaxNode>,
    eof: Token,
}

pub fn debug_ast(ast: &Ast, context: &Context) {
    for statement in &ast.statements {
        debug_syntax_node(statement, context, String::new());
    }
}

fn debug_syntax_node(node: &SyntaxNode, context: &Context, indent: String) {
    print!("{indent}");
    match &node.kind {
        SyntaxNodeKind::StylingStatement {
            name, type_, body, ..
        } => {
            println!(
                "Styling {} for {}:",
                name.text(context),
                type_.text(context)
            );
            for statement in body {
                debug_syntax_node(statement, context, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::ExpressionStatement { expression, .. } => {
            println!("ExpressionStatement:");
            debug_syntax_node(&expression, context, format!("{indent}    "));
        }
        SyntaxNodeKind::VariableDeclaration {
            name, expression, ..
        } => {
            println!("Variable Declaration {}:", name.text(context));
            debug_syntax_node(&expression, context, format!("{indent}    "));
        }
        SyntaxNodeKind::SlideStatement { name, body, .. } => {
            println!("Slide Declaration {}:", name.text(context));
            for statement in body {
                debug_syntax_node(statement, context, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::VariableReference { variable } => {
            println!("Variable {}", variable.text(context));
        }
        SyntaxNodeKind::Literal { literal } => {
            println!("Literal {}", literal.text(context));
        }
        SyntaxNodeKind::MemberAccess { base, member, .. } => {
            println!("Member Access:");
            debug_syntax_node(base, context, format!("{indent}    "));
            println!("{indent}    .{}", member.text(context));
        }
        SyntaxNodeKind::AssignmentStatement {
            expression,
            assignment,
            ..
        } => {
            println!("Assignment Statement:");
            debug_syntax_node(&expression, context, format!("{indent}    "));
            debug_syntax_node(&assignment, context, format!("{indent}    = "));
        }
        SyntaxNodeKind::FunctionCall {
            base, arguments, ..
        } => {
            println!("Function Call:");
            debug_syntax_node(&base, context, format!("{indent}    "));
            println!("{indent}    Arguments:");
            for (argument, _) in arguments {
                debug_syntax_node(&argument, context, format!("{indent}        "));
            }
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn match_token(&mut self, expected: TokenKind) -> Token {
        if self.current_token().kind == expected {
            self.next_token()
        } else {
            Token::fabricate(expected, self.current_token().location)
        }
    }

    fn next_token(&mut self) -> Token {
        self.index += 1;
        self.tokens[self.index - 1]
    }

    fn peek(&self) -> TokenKind {
        let next = (self.index + 1).min(self.tokens.len() - 1);
        self.tokens[next].kind
    }

    fn position(&self) -> usize {
        self.index
    }

    fn ensure_consume(&mut self, position: usize) {
        if self.index == position {
            self.next_token();
        }
    }

    fn try_match_token(&mut self, expected: TokenKind) -> Option<Token> {
        if self.current_token().kind == expected {
            Some(self.next_token())
        } else {
            None
        }
    }
}

pub(crate) fn parse_file(file: FileId, context: &mut Context) -> Ast {
    let tokens = lexer::lex(file, context);
    lexer::debug_tokens(&tokens, context);
    parse_tokens(tokens, context)
}

fn parse_tokens(tokens: Vec<Token>, context: &mut Context) -> Ast {
    let mut parser = Parser::new(tokens);

    parse_presentation(&mut parser, context)
}

fn parse_presentation(parser: &mut Parser, context: &mut Context) -> Ast {
    let mut statements = Vec::new();
    while parser.current_token().kind != TokenKind::Eof {
        let start = parser.position();
        statements.push(parse_top_level_statement(parser, context));
        parser.ensure_consume(start);
    }
    let eof = parser.match_token(TokenKind::Eof);
    Ast { statements, eof }
}

fn parse_top_level_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    match parser.current_token().kind {
        TokenKind::SlideKeyword => parse_slide_statement(parser, context),
        TokenKind::StylingKeyword => parse_styling_statement(parser, context),
        _ => todo!("Create error messages!"),
    }
}

fn parse_slide_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let slide_keyword = parser.match_token(TokenKind::SlideKeyword);
    let name = parser.match_token(TokenKind::Identifier);
    let colon = parser.match_token(TokenKind::SingleChar(':'));
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        body.push(parse_statement(parser, context));
    }

    SyntaxNode::slide_statement(slide_keyword, name, colon, body)
}

fn parse_styling_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let styling_keyword = parser.match_token(TokenKind::StylingKeyword);
    let name = parser.match_token(TokenKind::Identifier);
    let lparen = parser.match_token(TokenKind::SingleChar('('));
    let type_ = parser.match_token(TokenKind::Identifier);
    let rparen = parser.match_token(TokenKind::SingleChar(')'));
    let colon = parser.match_token(TokenKind::SingleChar(':'));
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        body.push(parse_statement(parser, context));
    }

    SyntaxNode::styling_statement(styling_keyword, name, lparen, type_, rparen, colon, body)
}

fn parse_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    match parser.current_token().kind {
        TokenKind::LetKeyword => parse_variable_declaration(parser, context),
        _ => parse_assignment_statemnt(parser, context),
    }
}

fn parse_assignment_statemnt(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let expression = parse_expression(parser, context);
    if parser.current_token().kind == TokenKind::SingleChar('=') {
        let equals = parser.next_token();
        let assignment = parse_expression(parser, context);
        let semicolon = parser.match_token(TokenKind::SingleChar(';'));

        SyntaxNode::assignment_statement(expression, equals, assignment, semicolon)
    } else {
        let semicolon = parser.match_token(TokenKind::SingleChar(';'));

        SyntaxNode::expression_statement(expression, semicolon)
    }
}

fn parse_variable_declaration(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let let_keyword = parser.match_token(TokenKind::LetKeyword);
    let name = parser.match_token(TokenKind::Identifier);
    let equals = parser.match_token(TokenKind::SingleChar('='));
    let expression = parse_expression(parser, context);
    let semicolon = parser.match_token(TokenKind::SingleChar(';'));

    SyntaxNode::variable_declaration(let_keyword, name, equals, expression, semicolon)
}

fn parse_expression(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    parse_function_call(parser, context)
}

fn parse_function_call(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let mut base = parse_primary(parser, context);
    loop {
        match parser.current_token().kind {
            TokenKind::SingleChar('(') => {
                let lparen = parser.next_token();
                let mut arguments = Vec::new();
                while parser.current_token().kind != TokenKind::SingleChar(')') {
                    let start = parser.position();

                    let argument = parse_expression(parser, context);
                    let optional_comma = parser.try_match_token(TokenKind::SingleChar(','));

                    arguments.push((argument, optional_comma));

                    parser.ensure_consume(start);
                }
                let rparen = parser.match_token(TokenKind::SingleChar(')'));
                base = SyntaxNode::function_call(base, lparen, arguments, rparen);
            }
            TokenKind::SingleChar('[') => todo!(),
            TokenKind::SingleChar('.') => {
                let period = parser.next_token();
                let member = parser.match_token(TokenKind::Identifier);
                base = SyntaxNode::member_access(base, period, member);
            }
            _ => break,
        }
    }
    base
}

fn parse_primary(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    match parser.current_token().kind {
        TokenKind::Identifier => SyntaxNode::variable_reference(parser.next_token()),
        TokenKind::Number => SyntaxNode::literal(parser.next_token()),
        err => {
            println!("HELLO");
            debug_tokens(&parser.tokens[parser.index..], context);
            todo!("error generation: {err:?}")
        }
    }
}

fn parse_member_access(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    todo!()
}

fn is_start_of_top_level_statement(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::SlideKeyword | TokenKind::StylingKeyword | TokenKind::Eof
    )
}
