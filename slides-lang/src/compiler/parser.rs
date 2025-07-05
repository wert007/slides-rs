use super::{
    diagnostics::Diagnostics,
    lexer::{self, Token, TokenKind},
};
use crate::{Context, FileId, Files, Location};

#[derive(Debug, Clone)]
pub struct StylingStatement {
    pub styling_keyword: Token,
    pub name: Token,
    pub lparen: Token,
    pub type_: Token,
    pub rparen: Token,
    pub colon: Token,
    pub body: Vec<SyntaxNode>,
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expression: Box<SyntaxNode>,
    pub semicolon: Token,
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub let_keyword: Token,
    pub name: Token,
    pub equals: Token,
    pub expression: Box<SyntaxNode>,
    pub semicolon: Token,
}

#[derive(Debug, Clone)]
pub struct SlideStatement {
    pub slide_keyword: Token,
    pub name: Token,
    pub colon: Token,
    pub body: Vec<SyntaxNode>,
}

#[derive(Debug, Clone)]
pub struct GlobalStatement {
    pub global_keyword: Token,
    pub colon: Token,
    pub body: Vec<SyntaxNode>,
}

#[derive(Debug, Clone)]
pub struct MemberAccess {
    pub base: Box<SyntaxNode>,
    pub period: Token,
    pub member: Token,
}

#[derive(Debug, Clone)]
pub struct AssignmentStatement {
    pub lhs: Box<SyntaxNode>,
    pub equals: Token,
    pub assignment: Box<SyntaxNode>,
    pub semicolon: Token,
}

#[derive(Debug, Clone)]

pub struct FunctionCall {
    pub base: Box<SyntaxNode>,
    pub lparen: Token,
    pub arguments: Vec<(SyntaxNode, Option<Token>)>,
    pub rparen: Token,
}

#[derive(Debug, Clone)]
pub struct TypedString {
    pub type_: Token,
    pub string: Token,
}

#[derive(Debug, Clone)]
pub struct DictEntry {
    pub identifier: Token,
    pub colon: Token,
    pub value: Box<SyntaxNode>,
}

#[derive(Debug, Clone)]
pub struct Dict {
    pub lbrace: Token,
    pub entries: Vec<(SyntaxNode, Option<Token>)>,
    pub rbrace: Token,
}

#[derive(Debug, Clone)]
pub struct Array {
    pub lbracket: Token,
    pub entries: Vec<(SyntaxNode, Option<Token>)>,
    pub rbracket: Token,
}

#[derive(Debug, Clone)]
pub struct ArrayAccess {
    pub base: Box<SyntaxNode>,
    pub lbracket: Token,
    pub index: Box<SyntaxNode>,
    pub rbracket: Token,
}

#[derive(Debug, Clone)]
pub struct InferredMember {
    pub period: Token,
    pub member: Token,
}

#[derive(Debug, Clone)]
pub struct PostInitialization {
    pub expression: Box<SyntaxNode>,
    pub dict: Box<SyntaxNode>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub identifier: Token,
    pub colon: Token,
    pub type_: Token,
    pub optional_equals: Option<Token>,
    pub optional_initializer: Option<Box<SyntaxNode>>,
}
#[derive(Debug, Clone)]
pub struct ParameterBlock {
    pub lparen: Token,
    pub parameters: Vec<(SyntaxNode, Option<Token>)>,
    pub rparen: Token,
}

#[derive(Debug, Clone)]
pub struct ElementStatement {
    pub element_keyword: Token,
    pub name: Token,
    pub parameters: Box<SyntaxNode>,
    pub colon: Token,
    pub body: Vec<SyntaxNode>,
}

#[derive(Debug, Clone)]
pub struct TemplateStatement {
    pub template_keyword: Token,
    pub name: Token,
    pub parameters: Box<SyntaxNode>,
    pub colon: Token,
    pub body: Vec<SyntaxNode>,
}

#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub import_keyword: Token,
    pub path: Box<SyntaxNode>,
    pub semicolon: Token,
}
#[derive(Debug, Clone)]
pub struct Binary {
    pub lhs: Box<SyntaxNode>,
    pub operator: Token,
    pub rhs: Box<SyntaxNode>,
}

#[derive(strum::EnumTryAs, Debug, strum::AsRefStr, Clone)]
pub enum SyntaxNodeKind {
    Error(bool),
    StylingStatement(StylingStatement),
    SlideStatement(SlideStatement),
    GlobalStatement(GlobalStatement),
    ElementStatement(ElementStatement),
    ImportStatement(ImportStatement),
    TemplateStatement(TemplateStatement),
    ExpressionStatement(ExpressionStatement),
    VariableDeclaration(VariableDeclaration),
    AssignmentStatement(AssignmentStatement),
    VariableReference(Token),
    Literal(Token),
    MemberAccess(MemberAccess),
    FunctionCall(FunctionCall),
    TypedString(TypedString),
    DictEntry(DictEntry),
    Dict(Dict),
    Array(Array),
    ArrayAccess(ArrayAccess),
    InferredMember(InferredMember),
    PostInitialization(PostInitialization),
    Parameter(Parameter),
    ParameterBlock(ParameterBlock),
    Binary(Binary),
    FormatString(Token),
}

#[derive(Debug, Clone)]
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

    fn global_statement(global_keyword: Token, colon: Token, body: Vec<SyntaxNode>) -> SyntaxNode {
        let location = Location::combine(
            global_keyword.location,
            body.last().expect("not empty").location,
        );
        SyntaxNode {
            kind: SyntaxNodeKind::GlobalStatement(GlobalStatement {
                global_keyword,
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

    fn format_string(string: Token) -> SyntaxNode {
        SyntaxNode {
            kind: SyntaxNodeKind::FormatString(string),
            location: string.location,
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

    fn error(token: Token, consumed: bool) -> SyntaxNode {
        SyntaxNode {
            location: token.location,
            kind: SyntaxNodeKind::Error(consumed),
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

    fn array(
        lbracket: Token,
        entries: Vec<(SyntaxNode, Option<Token>)>,
        rbracket: Token,
    ) -> SyntaxNode {
        let location = Location::combine(lbracket.location, rbracket.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::Array(Array {
                lbracket,
                entries,
                rbracket,
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

    fn parameter(
        identifier: Token,
        colon: Token,
        type_: Token,
        optional_equals: Option<Token>,
        optional_initializer: Option<SyntaxNode>,
    ) -> SyntaxNode {
        let location = Location::combine(identifier.location, type_.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::Parameter(Parameter {
                identifier,
                colon,
                type_,
                optional_equals,
                optional_initializer: optional_initializer.map(Box::new),
            }),
        }
    }

    fn parameter_block(
        lparen: Token,
        parameters: Vec<(SyntaxNode, Option<Token>)>,
        rparen: Token,
    ) -> SyntaxNode {
        let location = Location::combine(lparen.location, rparen.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::ParameterBlock(ParameterBlock {
                lparen,
                parameters,
                rparen,
            }),
        }
    }

    fn element_statement(
        element_keyword: Token,
        name: Token,
        parameters: SyntaxNode,
        colon: Token,
        body: Vec<SyntaxNode>,
    ) -> SyntaxNode {
        let location = Location::combine(
            element_keyword.location,
            body.last()
                .expect("no empty statements are allowed!")
                .location,
        );
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::ElementStatement(ElementStatement {
                element_keyword,
                name,
                parameters: Box::new(parameters),
                colon,
                body,
            }),
        }
    }

    fn import_statement(import_keyword: Token, path: SyntaxNode, semicolon: Token) -> SyntaxNode {
        let location = Location::combine(import_keyword.location, semicolon.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::ImportStatement(ImportStatement {
                import_keyword,
                path: Box::new(path),
                semicolon,
            }),
        }
    }

    fn template_statement(
        template_keyword: Token,
        name: Token,
        parameters: SyntaxNode,
        colon: Token,
        body: Vec<SyntaxNode>,
    ) -> SyntaxNode {
        let location = Location::combine(template_keyword.location, body.last().unwrap().location);

        SyntaxNode {
            location,
            kind: SyntaxNodeKind::TemplateStatement(TemplateStatement {
                template_keyword,
                name,
                parameters: Box::new(parameters),
                colon,
                body,
            }),
        }
    }

    fn binary(lhs: SyntaxNode, operator: Token, rhs: SyntaxNode) -> SyntaxNode {
        let location = Location::combine(lhs.location, rhs.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::Binary(Binary {
                lhs: Box::new(lhs),
                operator,
                rhs: Box::new(rhs),
            }),
        }
    }

    fn array_access(
        base: SyntaxNode,
        lbracket: Token,
        index: SyntaxNode,
        rbracket: Token,
    ) -> SyntaxNode {
        let location = Location::combine(base.location, rbracket.location);
        SyntaxNode {
            location,
            kind: SyntaxNodeKind::ArrayAccess(ArrayAccess {
                base: Box::new(base),
                lbracket,
                index: Box::new(index),
                rbracket,
            }),
        }
    }
}

pub struct Ast {
    pub statements: Vec<SyntaxNode>,
    pub eof: Token,
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
        SyntaxNodeKind::ElementStatement(element_statement) => {
            println!("Custom Element {}", element_statement.name.text(files),);
            debug_syntax_node(
                &element_statement.parameters,
                files,
                format!("{indent}        "),
            );
            println!("{indent}    Body:");
            for statement in &element_statement.body {
                debug_syntax_node(statement, files, format!("{indent}        "));
            }
        }
        SyntaxNodeKind::TemplateStatement(template_statement) => {
            println!("Custom Template {}", template_statement.name.text(files),);
            debug_syntax_node(
                &template_statement.parameters,
                files,
                format!("{indent}        "),
            );
            println!("{indent}    Body:");
            for statement in &template_statement.body {
                debug_syntax_node(statement, files, format!("{indent}        "));
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
        SyntaxNodeKind::GlobalStatement(global_statement) => {
            println!("Global execution");
            for statement in &global_statement.body {
                debug_syntax_node(statement, files, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::VariableReference(variable) => {
            println!("Variable {}", variable.text(files));
        }
        SyntaxNodeKind::FormatString(token) => println!("Format String {}", token.text(files)),

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
        SyntaxNodeKind::ArrayAccess(array_access) => {
            println!("Array Access (Index):");
            debug_syntax_node(&array_access.index, files, format!("{indent}    "));
            println!("{indent}Array Access (Base):");
            debug_syntax_node(&array_access.base, files, format!("{indent}    "));
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
        SyntaxNodeKind::Error(_) => println!("Error Node"),
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
        SyntaxNodeKind::Parameter(parameter) => {
            println!(
                "{}: {}",
                parameter.identifier.text(files),
                parameter.type_.text(files)
            )
        }
        SyntaxNodeKind::ParameterBlock(parameter_block) => {
            println!("Parameters");
            for (parameter, _) in &parameter_block.parameters {
                debug_syntax_node(parameter, files, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::ImportStatement(import_statement) => {
            println!("Import");
            debug_syntax_node(&import_statement.path, files, format!("{indent}    "));
        }
        SyntaxNodeKind::Array(array) => {
            println!("Array");
            for (entry, _) in &array.entries {
                debug_syntax_node(entry, files, format!("{indent}    "));
            }
        }
        SyntaxNodeKind::Binary(binary) => {
            println!("Binary {}", binary.operator.text(files));
            debug_syntax_node(&binary.lhs, files, format!("{indent}    "));
            debug_syntax_node(&binary.rhs, files, format!("{indent}    "));
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

    fn match_token(&mut self, expected: TokenKind, diagnostics: &mut Diagnostics) -> Token {
        if self.current_token().kind == expected {
            self.next_token()
        } else {
            diagnostics.report_unexpected_token(*self.current_token(), expected);
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

    fn ensure_consume(&mut self, position: usize) -> Option<Token> {
        if self.index == position {
            Some(self.next_token())
        } else {
            None
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
    if context.debug.tokens {
        lexer::debug_tokens(&tokens, &context.loaded_files);
    }
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
        if let Some(consumed) = parser.ensure_consume(start) {
            statements.push(SyntaxNode::error(consumed, true));
        }
    }
    let eof = parser.match_token(TokenKind::Eof, &mut context.diagnostics);
    Ast { statements, eof }
}

fn parse_top_level_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    match parser.current_token().kind {
        TokenKind::SlideKeyword => parse_slide_statement(parser, context),
        TokenKind::StylingKeyword => parse_styling_statement(parser, context),
        TokenKind::ElementKeyword => parse_element_statement(parser, context),
        TokenKind::TemplateKeyword => parse_template_statement(parser, context),
        TokenKind::ImportKeyword => parse_import_statement(parser, context),
        TokenKind::GlobalKeyword => parse_global_statement(parser, context),
        _ => {
            context
                .diagnostics
                .report_invalid_top_level_statement(*parser.current_token(), &context.loaded_files);
            SyntaxNode::error(*parser.current_token(), false)
        }
    }
}

fn parse_template_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let template_keyword = parser.match_token(TokenKind::TemplateKeyword, &mut context.diagnostics);
    let name = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    let parameters = parse_parameter_node(parser, context);
    let colon = parser.match_token(TokenKind::SingleChar(':'), &mut context.diagnostics);
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        let position = parser.position();

        body.push(parse_statement(parser, context));
        if let Some(consumed) = parser.ensure_consume(position) {
            body.push(SyntaxNode::error(consumed, true));
        }
    }
    SyntaxNode::template_statement(template_keyword, name, parameters, colon, body)
}

fn parse_import_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let import_keyword = parser.match_token(TokenKind::ImportKeyword, &mut context.diagnostics);
    let type_ = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    let string = parser.match_token(TokenKind::String, &mut context.diagnostics);
    let path = SyntaxNode::typed_string(type_, string);
    let semicolon = parser.match_token(TokenKind::SingleChar(';'), &mut context.diagnostics);
    SyntaxNode::import_statement(import_keyword, path, semicolon)
}

fn parse_global_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let slide_keyword = parser.match_token(TokenKind::GlobalKeyword, &mut context.diagnostics);
    let colon = parser.match_token(TokenKind::SingleChar(':'), &mut context.diagnostics);
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        let position = parser.position();

        body.push(parse_statement(parser, context));
        if let Some(consumed) = parser.ensure_consume(position) {
            body.push(SyntaxNode::error(consumed, true));
        }
    }

    SyntaxNode::global_statement(slide_keyword, colon, body)
}

fn parse_element_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let element_keyword = parser.match_token(TokenKind::ElementKeyword, &mut context.diagnostics);
    let name = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    let parameters = parse_parameter_node(parser, context);
    let colon = parser.match_token(TokenKind::SingleChar(':'), &mut context.diagnostics);
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        let position = parser.position();

        body.push(parse_statement(parser, context));
        if let Some(consumed) = parser.ensure_consume(position) {
            body.push(SyntaxNode::error(consumed, true));
        }
    }
    SyntaxNode::element_statement(element_keyword, name, parameters, colon, body)
}

fn parse_parameter_node(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let lparen = parser.match_token(TokenKind::SingleChar('('), &mut context.diagnostics);
    let mut parameters = Vec::new();
    while parser.current_token().kind != TokenKind::Eof
        && parser.current_token().kind != TokenKind::SingleChar(')')
    {
        let position = parser.position();

        let identifier = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
        let colon = parser.match_token(TokenKind::SingleChar(':'), &mut context.diagnostics);
        let type_ = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
        let optional_equals = parser.try_match_token(TokenKind::SingleChar('='));
        let optional_initializer = if optional_equals.is_some() {
            Some(parse_expression(parser, context))
        } else {
            None
        };
        let optional_comma = parser.try_match_token(TokenKind::SingleChar(','));
        parameters.push((
            SyntaxNode::parameter(
                identifier,
                colon,
                type_,
                optional_equals,
                optional_initializer,
            ),
            optional_comma,
        ));
        if let Some(consumed) = parser.ensure_consume(position) {
            parameters.push((SyntaxNode::error(consumed, true), None));
        }
    }
    let rparen = parser.match_token(TokenKind::SingleChar(')'), &mut context.diagnostics);
    SyntaxNode::parameter_block(lparen, parameters, rparen)
}

fn parse_slide_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let slide_keyword = parser.match_token(TokenKind::SlideKeyword, &mut context.diagnostics);
    let name = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    let colon = parser.match_token(TokenKind::SingleChar(':'), &mut context.diagnostics);
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        let position = parser.position();

        body.push(parse_statement(parser, context));
        if let Some(consumed) = parser.ensure_consume(position) {
            body.push(SyntaxNode::error(consumed, true));
        }
    }

    SyntaxNode::slide_statement(slide_keyword, name, colon, body)
}

fn parse_styling_statement(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let styling_keyword = parser.match_token(TokenKind::StylingKeyword, &mut context.diagnostics);
    let name = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    let lparen = parser.match_token(TokenKind::SingleChar('('), &mut context.diagnostics);
    let type_ = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    let rparen = parser.match_token(TokenKind::SingleChar(')'), &mut context.diagnostics);
    let colon = parser.match_token(TokenKind::SingleChar(':'), &mut context.diagnostics);
    let mut body = Vec::new();
    while !is_start_of_top_level_statement(parser.current_token().kind) {
        let position = parser.position();
        body.push(parse_statement(parser, context));
        if let Some(consumed) = parser.ensure_consume(position) {
            body.push(SyntaxNode::error(consumed, true));
        }
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
        let semicolon = parser.match_token(TokenKind::SingleChar(';'), &mut context.diagnostics);

        SyntaxNode::assignment_statement(expression, equals, assignment, semicolon)
    } else {
        let semicolon = parser.match_token(TokenKind::SingleChar(';'), &mut context.diagnostics);

        SyntaxNode::expression_statement(expression, semicolon)
    }
}

fn parse_variable_declaration(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let let_keyword = parser.match_token(TokenKind::LetKeyword, &mut context.diagnostics);
    let name = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    let equals = parser.match_token(TokenKind::SingleChar('='), &mut context.diagnostics);
    let expression = parse_expression(parser, context);
    let semicolon = parser.match_token(TokenKind::SingleChar(';'), &mut context.diagnostics);

    SyntaxNode::variable_declaration(let_keyword, name, equals, expression, semicolon)
}

fn parse_expression(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    parse_mul_div(parser, context)
}

fn parse_mul_div(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let mut lhs = parse_add_minus(parser, context);
    while parser.current_token().kind == TokenKind::SingleChar('*')
        || parser.current_token().kind == TokenKind::SingleChar('/')
    {
        let operator = parser.next_token();
        let rhs = parse_add_minus(parser, context);
        lhs = SyntaxNode::binary(lhs, operator, rhs);
    }
    lhs
}

fn parse_add_minus(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let mut lhs = parse_and_or(parser, context);
    while parser.current_token().kind == TokenKind::SingleChar('+')
        || parser.current_token().kind == TokenKind::SingleChar('-')
    {
        let operator = parser.next_token();
        let rhs = parse_and_or(parser, context);
        lhs = SyntaxNode::binary(lhs, operator, rhs);
    }
    lhs
}

fn parse_and_or(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let mut lhs = parse_post_initialization(parser, context);
    while parser.current_token().kind == TokenKind::SingleChar('&')
        || parser.current_token().kind == TokenKind::SingleChar('|')
    {
        let operator = parser.next_token();
        let rhs = parse_post_initialization(parser, context);
        lhs = SyntaxNode::binary(lhs, operator, rhs);
    }
    lhs
}

fn parse_post_initialization(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
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

                    if let Some(consumed) = parser.ensure_consume(start) {
                        arguments.push((SyntaxNode::error(consumed, true), None));
                    }
                }
                let rparen =
                    parser.match_token(TokenKind::SingleChar(')'), &mut context.diagnostics);
                base = SyntaxNode::function_call(base, lparen, arguments, rparen);
            }
            TokenKind::SingleChar('[') => {
                let lbracket = parser.next_token();
                let index = parse_expression(parser, context);
                let rbracket =
                    parser.match_token(TokenKind::SingleChar(']'), &mut context.diagnostics);
                base = SyntaxNode::array_access(base, lbracket, index, rbracket);
            }
            TokenKind::SingleChar('.') => {
                let period = parser.next_token();
                let member = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
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
            if parser.peek() == TokenKind::String || parser.peek() == TokenKind::FormatString {
                SyntaxNode::typed_string(parser.next_token(), parser.next_token())
            } else {
                SyntaxNode::variable_reference(parser.next_token())
            }
        }
        TokenKind::Number => {
            let number = parser.next_token();
            if parser.current_token().kind == TokenKind::Identifier
                || parser.current_token().kind == TokenKind::SingleChar('%')
            {
                let unit = parser.next_token();
                match Token::combine(number, unit, TokenKind::StyleUnitLiteral) {
                    Ok(token) => SyntaxNode::literal(token),
                    Err(err) => SyntaxNode::error(err, true),
                }
            } else {
                SyntaxNode::literal(number)
            }
        }
        TokenKind::String => SyntaxNode::literal(parser.next_token()),
        TokenKind::FormatString => SyntaxNode::format_string(parser.next_token()),
        TokenKind::SingleChar('{') => parse_dict(parser, context),
        TokenKind::SingleChar('[') => parse_array(parser, context),
        TokenKind::SingleChar('.') => parse_inferred_member(parser, context),
        _ => {
            context
                .diagnostics
                .report_expected_expression(*parser.current_token(), &context.loaded_files);
            SyntaxNode::error(*parser.current_token(), false)
        }
    }
}

fn parse_inferred_member(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let period = parser.match_token(TokenKind::SingleChar('.'), &mut context.diagnostics);
    let member = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    SyntaxNode::inferred_member(period, member)
}

fn parse_dict(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let lbrace = parser.match_token(TokenKind::SingleChar('{'), &mut context.diagnostics);
    let mut entries = Vec::new();
    while parser.current_token().kind != TokenKind::SingleChar('}')
        && parser.current_token().kind != TokenKind::Eof
    {
        let position = parser.position();
        let dict_identifier = parse_dict_identifier(parser, context);
        let colon = parser.match_token(TokenKind::SingleChar(':'), &mut context.diagnostics);
        let value = parse_expression(parser, context);
        let optional_comma = parser.try_match_token(TokenKind::SingleChar(','));

        entries.push((
            SyntaxNode::dict_entry(dict_identifier, colon, value),
            optional_comma,
        ));
        if let Some(consumed) = parser.ensure_consume(position) {
            entries.push((SyntaxNode::error(consumed, true), None));
        }
    }
    let rbrace = parser.match_token(TokenKind::SingleChar('}'), &mut context.diagnostics);
    SyntaxNode::dict(lbrace, entries, rbrace)
}

fn parse_array(parser: &mut Parser, context: &mut Context) -> SyntaxNode {
    let lbracket = parser.match_token(TokenKind::SingleChar('['), &mut context.diagnostics);
    let mut entries = Vec::new();
    while parser.current_token().kind != TokenKind::SingleChar(']')
        && parser.current_token().kind != TokenKind::Eof
    {
        let position = parser.position();
        let value = parse_expression(parser, context);
        let optional_comma = parser.try_match_token(TokenKind::SingleChar(','));

        entries.push((value, optional_comma));
        if let Some(consumed) = parser.ensure_consume(position) {
            entries.push((SyntaxNode::error(consumed, true), None));
        }
    }
    let rbracket = parser.match_token(TokenKind::SingleChar(']'), &mut context.diagnostics);
    SyntaxNode::array(lbracket, entries, rbracket)
}

fn parse_dict_identifier(parser: &mut Parser, context: &mut Context) -> Token {
    let identifier = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    // while parser.current_token().kind == TokenKind::SingleChar('-') {
    //     // TODO: Ensure that `foo - bar` is not a valid dict identifier
    //     // TODO: Ensure that `foo-2` is parsed correctly!
    //     let minus_token = parser.match_token(TokenKind::SingleChar('-'), &mut context.diagnostics);
    //     identifier = match Token::combine(identifier, minus_token, TokenKind::Identifier) {
    //         Ok(it) => it,
    //         Err(it) => it,
    //     };
    //     let identifier_part = parser.match_token(TokenKind::Identifier, &mut context.diagnostics);
    //     identifier = match Token::combine(identifier, identifier_part, TokenKind::Identifier) {
    //         Ok(it) => it,
    //         Err(it) => it,
    //     };
    // }
    identifier
}

fn is_start_of_top_level_statement(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::SlideKeyword
            | TokenKind::StylingKeyword
            | TokenKind::Eof
            | TokenKind::ElementKeyword
            | TokenKind::TemplateKeyword
            | TokenKind::GlobalKeyword
    )
}

pub(crate) fn parse_node(location: Location, context: &mut Context) -> SyntaxNode {
    let tokens = lexer::lex_source(location, context);
    let mut parser = Parser::new(tokens);
    parse_expression(&mut parser, context)
}
