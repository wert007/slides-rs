use std::process::id;

use crate::compiler::lexer::{self, debug_tokens};

use super::{
    Context, FileId, Files,
    diagnostics::Location,
    lexer::{Token, TokenKind},
};

#[derive(Debug)]
pub struct StylingStatement {
    pub styling_keyword: Token,
    pub name: Token,
    pub lparen: Token,
    pub type_: Token,
    pub rparen: Token,
    pub colon: Token,
    pub body: Vec<SyntaxNode>,
}

#[derive(Debug)]
pub struct ExpressionStatement {
    pub expression: Box<SyntaxNode>,
    pub semicolon: Token,
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub let_keyword: Token,
    pub name: Token,
    pub equals: Token,
    pub expression: Box<SyntaxNode>,
    pub semicolon: Token,
}

#[derive(Debug)]
pub struct SlideStatement {
    pub slide_keyword: Token,
    pub name: Token,
    pub colon: Token,
    pub body: Vec<SyntaxNode>,
}

#[derive(Debug)]
pub struct MemberAccess {
    pub base: Box<SyntaxNode>,
    pub period: Token,
    pub member: Token,
}

#[derive(Debug)]
pub struct AssignmentStatement {
    pub lhs: Box<SyntaxNode>,
    pub equals: Token,
    pub assignment: Box<SyntaxNode>,
    pub semicolon: Token,
}

#[derive(Debug)]

pub struct FunctionCall {
    pub base: Box<SyntaxNode>,
    pub lparen: Token,
    pub arguments: Vec<(SyntaxNode, Option<Token>)>,
    pub rparen: Token,
}

#[derive(Debug)]
pub struct TypedString {
    pub type_: Token,
    pub string: Token,
}

#[derive(Debug)]
pub struct DictEntry {
    pub identifier: Token,
    pub colon: Token,
    pub value: Box<SyntaxNode>,
}

#[derive(Debug)]
pub struct Dict {
    pub lbrace: Token,
    pub entries: Vec<(SyntaxNode, Option<Token>)>,
    pub rbrace: Token,
}

#[derive(Debug)]
pub struct InferredMember {
    pub period: Token,
    pub member: Token,
}

#[derive(Debug)]
pub struct PostInitialization {
    pub expression: Box<SyntaxNode>,
    pub dict: Box<SyntaxNode>,
}

#[derive(strum::EnumTryAs, Debug)]
pub enum SyntaxNodeKind {
    StylingStatement(StylingStatement),
    ExpressionStatement(ExpressionStatement),
    VariableDeclaration(VariableDeclaration),
    SlideStatement(SlideStatement),
    VariableReference(Token),
    Literal(Token),
    MemberAccess(MemberAccess),
    AssignmentStatement(AssignmentStatement),
    FunctionCall(FunctionCall),
    TypedString(TypedString),
    Error,
    DictEntry(DictEntry),
    Dict(Dict),
    InferredMember(InferredMember),
    PostInitialization(PostInitialization),
}

#[derive(Debug)]
pub struct SyntaxNode {
    pub location: Location,
    pub kind: SyntaxNodeKind,
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
        let location = Location::combine(
            styling_keyword.location,
            body.last().expect("not be empty").location,
        );
        SyntaxNode {
            kind: SyntaxNodeKind::StylingStatement(StylingStatement {
                styling_keyword,
                name,
                lparen,
                type_,
                rparen,
                colon,
                body,
            }),
            location,
        }
    }

    fn expression_statement(expression: SyntaxNode, semicolon: Token) -> SyntaxNode {
        let location = Location::combine(expression.location, semicolon.location);
        SyntaxNode {
            kind: SyntaxNodeKind::ExpressionStatement(ExpressionStatement {
                expression: Box::new(expression),
                semicolon,
            }),
            location,
        }
    }

    fn variable_declaration(
        let_keyword: Token,
        name: Token,
        equals: Token,
        expression: SyntaxNode,
        semicolon: Token,
    ) -> SyntaxNode {
        let location = Location::combine(let_keyword.location, semicolon.location);
        SyntaxNode {
            kind: SyntaxNodeKind::VariableDeclaration(VariableDeclaration {
                let_keyword,
                name,
                equals,
                expression: Box::new(expression),
                semicolon,
            }),
            location,
        }
    }

    fn slide_statement(
        slide_keyword: Token,
        name: Token,
        colon: Token,
        body: Vec<SyntaxNode>,
    ) -> SyntaxNode {
        let location = Location::combine(
            slide_keyword.location,
            body.last().expect("not empty").location,
        );
        SyntaxNode {
            kind: SyntaxNodeKind::SlideStatement(SlideStatement {
                slide_keyword,
                name,
                colon,
                body,
            }),
            location,
        }
    }

    fn variable_reference(variable: Token) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::VariableReference(variable),
            location: variable.location,
        }
    }

    fn literal(literal: Token) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::Literal(literal),
            location: literal.location,
        }
    }

    fn member_access(base: SyntaxNode, period: Token, member: Token) -> SyntaxNode {
        let location = Location::combine(base.location, member.location);
        SyntaxNode {
            kind: SyntaxNodeKind::MemberAccess(MemberAccess {
                base: Box::new(base),
                period,
                member,
            }),
            location,
        }
    }

    fn assignment_statement(
        expression: SyntaxNode,
        equals: Token,
        assignment: SyntaxNode,
        semicolon: Token,
    ) -> SyntaxNode {
        let location = Location::combine(expression.location, semicolon.location);
        SyntaxNode {
            kind: SyntaxNodeKind::AssignmentStatement(AssignmentStatement {
                lhs: Box::new(expression),
                equals,
                assignment: Box::new(assignment),
                semicolon,
            }),
            location,
        }
    }

    fn function_call(
        base: SyntaxNode,
        lparen: Token,
        arguments: Vec<(SyntaxNode, Option<Token>)>,
        rparen: Token,
    ) -> SyntaxNode {
        let location = Location::combine(base.location, rparen.location);
        SyntaxNode {
            kind: SyntaxNodeKind::FunctionCall(FunctionCall {
                base: Box::new(base),
                lparen,
                arguments,
                rparen,
            }),
            location,
        }
    }

    fn typed_string(type_: Token, string: Token) -> SyntaxNode {
        let location = Location::combine(type_.location, string.location);
        SyntaxNode {
            kind: SyntaxNodeKind::TypedString(TypedString { type_, string }),
            location,
        }
    }

    fn error(token: Token) -> SyntaxNode {
        SyntaxNode {
            location: token.location,
            kind: SyntaxNodeKind::Error,
        }
    }

    fn dict_entry(identifier: Token, colon: Token, value: SyntaxNode) -> Self {
        let location = Location::combine(identifier.location, value.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::DictEntry(DictEntry {
                identifier,
                colon,
                value: Box::new(value),
            }),
        }
    }

    fn dict(lbrace: Token, entries: Vec<(SyntaxNode, Option<Token>)>, rbrace: Token) -> SyntaxNode {
        let location = Location::combine(lbrace.location, rbrace.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::Dict(Dict {
                lbrace,
                entries,
                rbrace,
            }),
        }
    }

    fn inferred_member(period: Token, member: Token) -> SyntaxNode {
        let location = Location::combine(period.location, member.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::InferredMember(InferredMember { period, member }),
        }
    }

    fn post_initialization(expression: SyntaxNode, dict: SyntaxNode) -> SyntaxNode {
        let location = Location::combine(expression.location, dict.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::PostInitialization(PostInitialization {
                expression: Box::new(expression),
                dict: Box::new(dict),
            }),
        }
    }
}

pub struct Ast {
    pub statements: Vec<SyntaxNode>,
    eof: Token,
}

pub fn debug_ast(ast: &Ast, context: &Context) {
    for statement in &ast.statements {
        debug_syntax_node(statement, &context.loaded_files, String::new());
    }
}

fn debug_syntax_node(node: &SyntaxNode, files: &Files, indent: String) {
    print!("{indent}");
    match &node.kind {
        SyntaxNodeKind::StylingStatement(styling_statement) => {
            println!(
                "Styling {} for {}:",
                styling_statement.name.text(files),
                styling_statement.type_.text(files)
            );
            for statement in &styling_statement.body {
                debug_syntax_node(statement, files, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::ExpressionStatement(expression_statement) => {
            println!("ExpressionStatement:");
            debug_syntax_node(
                &expression_statement.expression,
                files,
                format!("{indent}    "),
            );
        }
        SyntaxNodeKind::VariableDeclaration(variable_declaration) => {
            println!(
                "Variable Declaration {}:",
                variable_declaration.name.text(files)
            );
            debug_syntax_node(
                &variable_declaration.expression,
                files,
                format!("{indent}    "),
            );
        }
        SyntaxNodeKind::SlideStatement(slide_statement) => {
            println!("Slide Declaration {}:", slide_statement.name.text(files));
            for statement in &slide_statement.body {
                debug_syntax_node(statement, files, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::VariableReference(variable) => {
            println!("Variable {}", variable.text(files));
        }
        SyntaxNodeKind::Literal(literal) => {
            println!("Literal {}", literal.text(files));
        }
        SyntaxNodeKind::MemberAccess(member_access) => {
            println!("Member Access:");
            debug_syntax_node(&member_access.base, files, format!("{indent}    "));
            println!("{indent}    .{}", member_access.member.text(files));
        }
        SyntaxNodeKind::AssignmentStatement(assignment_statement) => {
            println!("Assignment Statement:");
            debug_syntax_node(&assignment_statement.lhs, files, format!("{indent}    "));
            debug_syntax_node(
                &assignment_statement.assignment,
                files,
                format!("{indent}    = "),
            );
        }
        SyntaxNodeKind::FunctionCall(function_call) => {
            println!("Function Call:");
            debug_syntax_node(&function_call.base, files, format!("{indent}    "));
            println!("{indent}    Arguments:");
            for (argument, _) in &function_call.arguments {
                debug_syntax_node(&argument, files, format!("{indent}        "));
            }
        }
        SyntaxNodeKind::TypedString(typed_string) => {
            println!(
                "Typed String {}{}",
                typed_string.type_.text(files),
                typed_string.string.text(files)
            );
        }
        SyntaxNodeKind::Error => println!("Error Node"),
        SyntaxNodeKind::DictEntry(dict_entry) => {
            println!("DictEntry");
            debug_syntax_node(
                &dict_entry.value,
                files,
                format!("{indent}    {}: ", dict_entry.identifier.text(files)),
            );
        }
        SyntaxNodeKind::Dict(dict) => {
            println!("Dict");
            for (entry, _) in &dict.entries {
                debug_syntax_node(entry, files, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::InferredMember(inferred_member) => {
            println!("Inferred member {}", inferred_member.member.text(files))
        }
        SyntaxNodeKind::PostInitialization(post_initialization) => {
            println!("Post Initialized Expression");
            debug_syntax_node(
                &post_initialization.expression,
                files,
                format!("{indent}    "),
            );
            debug_syntax_node(&post_initialization.dict, files, format!("{indent}    "));
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
    lexer::debug_tokens(&tokens, &context.loaded_files);
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
        _ => {
            context
                .diagnostics
                .report_invalid_top_level_statement(*parser.current_token(), &context.loaded_files);
            SyntaxNode::error(*parser.current_token())
        }
    }
}

fn parse_slide_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let slide_keyword = parser.match_token(TokenKind::SlideKeyword);
    let name = parser.match_token(TokenKind::Identifier);
    let colon = parser.match_token(TokenKind::SingleChar(':'));
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        let position = parser.position();

        body.push(parse_statement(parser, context));
        parser.ensure_consume(position);
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
        let position = parser.position();
        body.push(parse_statement(parser, context));
        parser.ensure_consume(position);
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
    let expression = parse_function_call(parser, context);
    if parser.current_token().kind == TokenKind::SingleChar('{') {
        let dict = parse_dict(parser, context);
        SyntaxNode::post_initialization(expression, dict)
    } else {
        expression
    }
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
        TokenKind::Identifier => {
            if parser.peek() == TokenKind::String {
                SyntaxNode::typed_string(parser.next_token(), parser.next_token())
            } else {
                SyntaxNode::variable_reference(parser.next_token())
            }
        }
        TokenKind::Number => SyntaxNode::literal(parser.next_token()),
        TokenKind::String => SyntaxNode::literal(parser.next_token()),
        TokenKind::SingleChar('{') => parse_dict(parser, context),
        TokenKind::SingleChar('.') => parse_inferred_member(parser, context),
        _ => {
            debug_tokens(&parser.tokens[parser.index..], &context.loaded_files);
            context
                .diagnostics
                .report_expected_expression(*parser.current_token(), &context.loaded_files);
            SyntaxNode::error(*parser.current_token())
        }
    }
}

fn parse_inferred_member(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let period = parser.match_token(TokenKind::SingleChar('.'));
    let member = parser.match_token(TokenKind::Identifier);
    SyntaxNode::inferred_member(period, member)
}

fn parse_dict(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let lbrace = parser.match_token(TokenKind::SingleChar('{'));
    let mut entries = Vec::new();
    while parser.current_token().kind != TokenKind::SingleChar('}')
        && parser.current_token().kind != TokenKind::Eof
    {
        let position = parser.position();
        let dict_identifier = parse_dict_identifier(parser, context);
        let colon = parser.match_token(TokenKind::SingleChar(':'));
        let value = parse_expression(parser, context);
        let optional_comma = parser.try_match_token(TokenKind::SingleChar(','));

        entries.push((
            SyntaxNode::dict_entry(dict_identifier, colon, value),
            optional_comma,
        ));
        parser.ensure_consume(position);
    }
    let rbrace = parser.match_token(TokenKind::SingleChar('}'));
    SyntaxNode::dict(lbrace, entries, rbrace)
}

fn parse_dict_identifier(parser: &mut Parser, context: &mut Context) -> Token {
    let mut identifier = parser.match_token(TokenKind::Identifier);
    while parser.current_token().kind == TokenKind::SingleChar('-') {
        // TODO: Ensure that `foo - bar` is not a valid dict identifier
        // TODO: Ensure that `foo-2` is parsed correctly!
        let _minus_token = parser.match_token(TokenKind::SingleChar('-'));
        let identifier_part = parser.match_token(TokenKind::Identifier);

        identifier = Token {
            location: Location::combine(identifier.location, identifier_part.location),
            kind: TokenKind::Identifier,
        };
    }
    identifier
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
