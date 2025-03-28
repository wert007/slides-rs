use std::{collections::HashMap, path::PathBuf};

use slides_rs_core::Presentation;
use string_interner::{Symbol, symbol::SymbolUsize};

use super::{
    Context,
    diagnostics::Location,
    lexer::Token,
    parser::{self, SyntaxNode, SyntaxNodeKind, debug_ast},
};

pub(crate) fn create_presentation_from_file(file: PathBuf) -> slides_rs_core::Result<Presentation> {
    let mut context = Context::new();
    let file = context.load_file(file)?;
    let ast = parser::parse_file(file, &mut context);
    debug_ast(&ast, &context);
    let ast = bind_ast(ast, &mut context);
    debug_bound_ast(&ast, &context);
    let Context {
        presentation,
        diagnostics,
        loaded_files,
        ..
    } = context;
    if !diagnostics.is_empty() {
        diagnostics.write(&mut std::io::stdout(), &loaded_files)?;
    }
    Ok(presentation)
}

fn debug_bound_ast(ast: &BoundAst, context: &Context) {
    for statement in &ast.statements {
        debug_bound_node(statement, context, String::new());
    }
}

fn debug_bound_node(statement: &BoundNode, context: &Context, indent: String) {
    print!("{indent}");
    match &statement.kind {
        BoundNodeKind::Error => println!("#Error"),
        BoundNodeKind::StylingStatement(styling_statement) => {
            println!(
                "Style {} for {:?}",
                context
                    .string_interner
                    .resolve_variable(styling_statement.name),
                styling_statement.type_
            );
            for statement in &styling_statement.body {
                debug_bound_node(statement, context, format!("{indent}    "));
            }
        }
        BoundNodeKind::AssignmentStatement(assignment_statement) => {
            println!("Assignment");
            debug_bound_node(&assignment_statement.lhs, context, format!("{indent}    "));
            debug_bound_node(
                &assignment_statement.value,
                context,
                format!("{indent}    "),
            );
        }
        BoundNodeKind::FunctionCall(function_call) => {
            println!("FunctionCall");
            debug_bound_node(&function_call.base, context, format!("{indent}    "));
            for arg in &function_call.arguments {
                debug_bound_node(arg, context, format!("{indent}        "));
            }
        }
        BoundNodeKind::VariableReference(variable) => {
            println!(
                "Variable {}: {:?}",
                context.string_interner.resolve_variable(variable.id),
                variable.type_
            );
        }
        BoundNodeKind::Literal(value) => {
            println!("Literal {value:?}");
        }
        BoundNodeKind::SlideStatement(slide_statement) => {
            println!(
                "Slide {}",
                context
                    .string_interner
                    .resolve_variable(slide_statement.name)
            );
            for statement in &slide_statement.body {
                debug_bound_node(statement, context, format!("{indent}    "));
            }
        }
        BoundNodeKind::VariableDeclaration(variable_declaration) => {
            println!(
                "Variable Declaration {}",
                context
                    .string_interner
                    .resolve_variable(variable_declaration.variable)
            );
            debug_bound_node(
                &variable_declaration.value,
                context,
                format!("{indent}    ="),
            );
        }
        BoundNodeKind::Dict(items) => todo!(),
        BoundNodeKind::MemberAccess(member_access) => {
            println!(
                "Member Access .{}",
                context.string_interner.resolve(member_access.member)
            );
            debug_bound_node(&member_access.base, context, format!("{indent}    "));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub id: VariableId,
    pub definition: Location,
    pub type_: Type,
}

struct Scope {
    variables: HashMap<VariableId, Variable>,
}

impl Scope {
    pub fn global(interner: &mut super::StringInterner) -> Self {
        let mut global = Self {
            variables: HashMap::new(),
        };
        let id = interner.create_or_get_variable("rgb");
        global.try_register_variable(
            id,
            Type::Function(FunctionType {
                argument_types: vec![Type::Integer, Type::Integer, Type::Integer],
                return_type: Box::new(Type::Color),
            }),
            Location::zero(),
        );
        global
    }

    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    fn try_register_variable(
        &mut self,
        id: VariableId,
        type_: Type,
        definition: Location,
    ) -> Result<&Variable, &Variable> {
        if self.variables.contains_key(&id) {
            Err(&self.variables[&id])
        } else {
            self.variables.insert(
                id,
                Variable {
                    id,
                    definition,
                    type_,
                },
            );
            Ok(&self.variables[&id])
        }
    }

    fn look_up(&self, variable_id: VariableId) -> Option<&Variable> {
        self.variables.get(&variable_id)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct VariableId(usize);

impl Symbol for VariableId {
    fn try_from_usize(index: usize) -> Option<Self> {
        Some(Self(index))
    }

    fn to_usize(self) -> usize {
        self.0
    }
}

struct Binder {
    scopes: Vec<Scope>,
}

impl Binder {
    pub fn new(interner: &mut super::StringInterner) -> Self {
        Self {
            scopes: vec![Scope::global(interner)],
        }
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("There is at least one scope")
    }

    fn expect_register_variable_token(
        &mut self,
        token: Token,
        type_: Type,
        location: Location,
        context: &mut Context,
    ) -> Option<VariableId> {
        let name = token.text(&context.loaded_files);
        let variable = context.string_interner.create_or_get_variable(name);
        self.expect_register_variable_id(variable, type_, location, context)
    }

    fn expect_register_variable_id(
        &mut self,
        variable: VariableId,
        type_: Type,
        location: Location,
        context: &mut Context,
        // name: &str,
    ) -> Option<VariableId> {
        match self
            .current_scope_mut()
            .try_register_variable(variable, type_, location)
        {
            Ok(_) => Some(variable),
            Err(previous) => {
                let name = context.string_interner.resolve_variable(variable);
                context
                    .diagnostics
                    .report_redeclaration_of_variable(location, name, previous);
                None
            }
        }
    }

    fn create_scope(&mut self) -> &mut Scope {
        self.scopes.push(Scope::new());
        self.scopes.last_mut().unwrap()
    }

    fn drop_scope(&mut self) {
        assert!(self.scopes.len() > 1);
        self.scopes.pop();
    }

    fn look_up_variable(&self, id: VariableId) -> Option<&Variable> {
        self.scopes
            .iter()
            .rev()
            .filter_map(|s| s.look_up(id))
            .next()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    argument_types: Vec<Type>,
    return_type: Box<Type>,
}
impl FunctionType {
    fn return_type(&self) -> Type {
        self.return_type.as_ref().clone()
    }
}

#[derive(Debug, strum::EnumTryAs, Clone)]
pub enum Type {
    Error,
    Void,
    Float,
    Integer,
    String,
    Dict,
    Styling,
    Background,
    Color,
    ObjectFit,
    Function(FunctionType),
    Slide,
}
impl Type {
    fn field_type(&self, member: &str) -> Option<Type> {
        match self {
            Type::Error => Some(Type::Error),
            Type::Void => None,
            Type::Float => None,
            Type::Integer => None,
            Type::String => None,
            Type::Dict => None,
            Type::Styling => None,
            Type::Background => None,
            Type::Color => None,
            Type::ObjectFit => None,
            Type::Function(_) => None,
            Type::Slide => None,
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Float(f64),
    Integer(i64),
    String(String),
}

impl Value {
    pub fn infer_type(&self) -> Type {
        match self {
            Value::Float(_) => Type::Float,
            Value::Integer(_) => Type::Integer,
            Value::String(_) => Type::String,
        }
    }

    fn parse_string_literal(text: &str) -> Value {
        let mut result = String::with_capacity(text.len());
        let mut tmp = text.chars();
        while let Some(ch) = tmp.next() {
            match ch {
                _ => result.push(ch),
            }
        }
        Value::String(result)
    }
}

#[derive(Debug, strum::EnumString)]
enum StylingType {
    Label,
    Image,
    Slide,
}

struct StylingStatement {
    name: VariableId,
    type_: StylingType,
    body: Vec<BoundNode>,
}

struct AssignmentStatement {
    lhs: Box<BoundNode>,
    value: Box<BoundNode>,
}

struct FunctionCall {
    base: Box<BoundNode>,
    arguments: Vec<BoundNode>,
    function_type: FunctionType,
}

struct SlideStatement {
    name: VariableId,
    body: Vec<BoundNode>,
}

struct VariableDeclaration {
    variable: VariableId,
    value: Box<BoundNode>,
}

struct MemberAccess {
    base: Box<BoundNode>,
    member: SymbolUsize,
}

enum BoundNodeKind {
    Error,
    StylingStatement(StylingStatement),
    AssignmentStatement(AssignmentStatement),
    FunctionCall(FunctionCall),
    VariableReference(Variable),
    Literal(Value),
    SlideStatement(SlideStatement),
    VariableDeclaration(VariableDeclaration),
    Dict(Vec<(String, BoundNode)>),
    MemberAccess(MemberAccess),
}

struct BoundNode {
    base: Option<SyntaxNodeKind>,
    location: Location,
    kind: BoundNodeKind,
    type_: Type,
}
impl BoundNode {
    fn syntax_error(location: Location) -> BoundNode {
        BoundNode {
            base: Some(SyntaxNodeKind::Error),
            location,
            kind: BoundNodeKind::Error,
            type_: Type::Error,
        }
    }

    fn error(location: Location) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::Error,
            type_: Type::Error,
        }
    }

    fn styling_statement(
        base: parser::StylingStatement,
        location: Location,
        name: VariableId,
        type_: StylingType,
        body: Vec<BoundNode>,
    ) -> BoundNode {
        BoundNode {
            base: Some(SyntaxNodeKind::StylingStatement(base)),
            location,
            kind: BoundNodeKind::StylingStatement(StylingStatement { name, type_, body }),
            type_: Type::Void,
        }
    }

    fn assignment_statement(location: Location, lhs: BoundNode, value: BoundNode) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::AssignmentStatement(AssignmentStatement {
                lhs: Box::new(lhs),
                value: Box::new(value),
            }),
            type_: Type::Void,
        }
    }

    fn function_call(
        location: Location,
        base: BoundNode,
        arguments: Vec<BoundNode>,
        function_type: FunctionType,
    ) -> BoundNode {
        let type_ = function_type.return_type();
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::FunctionCall(FunctionCall {
                base: Box::new(base),
                arguments,
                function_type,
            }),
            type_,
        }
    }

    fn variable_reference(token: super::lexer::Token, variable: &Variable) -> BoundNode {
        BoundNode {
            base: Some(SyntaxNodeKind::VariableReference(token)),
            location: token.location,
            kind: BoundNodeKind::VariableReference(variable.clone()),
            type_: variable.type_.clone(),
        }
    }

    fn literal(token: super::lexer::Token, value: Value) -> BoundNode {
        let type_ = value.infer_type();
        BoundNode {
            base: Some(SyntaxNodeKind::Literal(token)),
            location: token.location,
            kind: BoundNodeKind::Literal(value),
            type_,
        }
    }

    fn slide_statement(
        slide_statement: parser::SlideStatement,
        location: Location,
        name: VariableId,
        body: Vec<BoundNode>,
    ) -> BoundNode {
        BoundNode {
            base: Some(SyntaxNodeKind::SlideStatement(slide_statement)),
            location,
            kind: BoundNodeKind::SlideStatement(SlideStatement { name, body }),
            type_: Type::Void,
        }
    }

    fn variable_declaration(
        location: Location,
        variable: VariableId,
        value: BoundNode,
    ) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::VariableDeclaration(VariableDeclaration {
                variable,
                value: Box::new(value),
            }),
            type_: Type::Void,
        }
    }

    fn dict(
        dict: parser::Dict,
        location: Location,
        entries: Vec<(String, BoundNode)>,
    ) -> BoundNode {
        BoundNode {
            base: Some(SyntaxNodeKind::Dict(dict)),
            location,
            kind: BoundNodeKind::Dict(entries),
            type_: Type::Dict,
        }
    }

    fn member_access(
        location: Location,
        base: BoundNode,
        member: SymbolUsize,
        type_: Type,
    ) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::MemberAccess(MemberAccess {
                base: Box::new(base),
                member,
            }),
            type_,
        }
    }
}

struct BoundAst {
    statements: Vec<BoundNode>,
}

fn bind_ast(ast: parser::Ast, context: &mut Context) -> BoundAst {
    let mut binder = Binder::new(&mut context.string_interner);
    let mut statements = Vec::with_capacity(ast.statements.len());
    for statement in ast.statements {
        statements.push(bind_node(statement, &mut binder, context));
    }
    BoundAst { statements }
}

fn bind_node(statement: SyntaxNode, binder: &mut Binder, context: &mut Context) -> BoundNode {
    match statement.kind {
        SyntaxNodeKind::StylingStatement(styling_statement) => {
            bind_styling_statement(styling_statement, statement.location, binder, context)
        }
        SyntaxNodeKind::SlideStatement(slide_statement) => {
            bind_slide_statement(slide_statement, statement.location, binder, context)
        }
        SyntaxNodeKind::ExpressionStatement(expression_statement) => {
            let mut result = bind_node(*expression_statement.expression, binder, context);
            result.type_ = Type::Void;
            result
        }
        SyntaxNodeKind::VariableDeclaration(variable_declaration) => {
            bind_variable_declaration(variable_declaration, statement.location, binder, context)
        }
        SyntaxNodeKind::VariableReference(token) => bind_variable_reference(token, binder, context),
        SyntaxNodeKind::Literal(token) => bind_literal(token, binder, context),
        SyntaxNodeKind::MemberAccess(member_access) => {
            bind_member_access(member_access, statement.location, binder, context)
        }
        SyntaxNodeKind::AssignmentStatement(assignment_statement) => {
            bind_assignment_statement(assignment_statement, statement.location, binder, context)
        }
        SyntaxNodeKind::FunctionCall(function_call) => {
            bind_function_call(function_call, statement.location, binder, context)
        }
        SyntaxNodeKind::TypedString(typed_string) => {
            bind_typed_string(typed_string, statement.location, binder, context)
        }
        SyntaxNodeKind::Error => BoundNode::syntax_error(statement.location),
        SyntaxNodeKind::DictEntry(dict_entry) => todo!(),
        SyntaxNodeKind::Dict(dict) => bind_dict(dict, statement.location, binder, context),
        SyntaxNodeKind::InferredMember(inferred_member) => todo!(),
    }
}

fn bind_member_access(
    member_access: parser::MemberAccess,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let base = bind_node(*member_access.base, binder, context);
    let member = member_access.member.text(&context.loaded_files);
    if let Some(type_) = base.type_.field_type(member) {
        let member = context.string_interner.create_or_get(member);
        BoundNode::member_access(location, base, member, type_)
    } else {
        context
            .diagnostics
            .report_unknown_member(member_access.member, base.type_, member);
        BoundNode::error(location)
    }
}

fn bind_dict(
    mut dict: parser::Dict,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let mut entries = Vec::with_capacity(dict.entries.len());
    for (entry, _) in dict.entries {
        let entry = entry
            .kind
            .try_as_dict_entry()
            .expect("should not have parsed");
        let key = entry.identifier.text(&context.loaded_files).to_string();
        let value = bind_node(*entry.value, binder, context);
        entries.push((key, value));
    }
    dict.entries = Vec::new();
    BoundNode::dict(dict, location, entries)
}

fn bind_variable_declaration(
    variable_declaration: parser::VariableDeclaration,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let value = bind_node(*variable_declaration.expression, binder, context);
    let Some(variable) = binder.expect_register_variable_token(
        variable_declaration.name,
        value.type_.clone(),
        location,
        context,
    ) else {
        return BoundNode::error(location);
    };
    BoundNode::variable_declaration(location, variable, value)
}

fn bind_slide_statement(
    mut slide_statement: parser::SlideStatement,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let mut statements = Vec::with_capacity(slide_statement.body.len());
    for statement in slide_statement.body {
        statements.push(bind_node(statement, binder, context));
    }
    slide_statement.body = Vec::new();
    let Some(name) = binder.expect_register_variable_token(
        slide_statement.name,
        Type::Slide,
        slide_statement.name.location,
        context,
    ) else {
        return BoundNode::error(slide_statement.name.location);
    };
    BoundNode::slide_statement(slide_statement, location, name, statements)
}

fn bind_typed_string(
    typed_string: parser::TypedString,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let type_ = typed_string.type_.text(&context.loaded_files);
    match type_ {
        "c" | "l" => {
            let text = typed_string.string.text(&context.loaded_files);
            let text = &text[1..text.len() - 1];
            BoundNode::literal(typed_string.string, Value::parse_string_literal(text))
        }
        unknown => {
            context
                .diagnostics
                .report_unknown_string_type(unknown, location);
            return BoundNode::error(typed_string.type_.location);
        }
    }
}

fn bind_literal(
    token: super::lexer::Token,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let text = token.text(&context.loaded_files);
    let value = match token.kind {
        super::lexer::TokenKind::Number => {
            if text.contains('.') {
                Value::Float(text.parse().expect("lexer filtered"))
            } else {
                Value::Integer(text.parse().expect("lexer filtered"))
            }
        }
        super::lexer::TokenKind::String => {
            let text = &text[1..text.len() - 1];
            Value::parse_string_literal(text)
        }
        err => unreachable!("This is a unhandled literal {err:?}"),
    };
    BoundNode::literal(token, value)
}

fn bind_variable_reference(
    token: super::lexer::Token,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let name = context
        .string_interner
        .create_or_get_variable(token.text(&context.loaded_files));
    let Some(variable) = binder.look_up_variable(name) else {
        // TODO: Show error, that variable could not be found!
        context
            .diagnostics
            .report_unknown_variable(token.location, token.text(&context.loaded_files));
        return BoundNode::error(token.location);
    };
    BoundNode::variable_reference(token, variable)
}

fn bind_function_call(
    function_call: parser::FunctionCall,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let base = bind_node(*function_call.base, binder, context);
    let mut arguments = Vec::with_capacity(function_call.arguments.len());
    for (argument, _) in function_call.arguments {
        arguments.push(bind_node(argument, binder, context));
    }
    let Some(function_type) = base.type_.clone().try_as_function() else {
        // TODO: Report unexpected Type!
        return BoundNode::error(base.location);
    };
    BoundNode::function_call(location, base, arguments, function_type)
}

fn bind_assignment_statement(
    assignment_statement: parser::AssignmentStatement,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let lhs = bind_node(*assignment_statement.assignment, binder, context);
    let value = bind_node(*assignment_statement.expression, binder, context);
    // TODO: Type checking!
    BoundNode::assignment_statement(location, lhs, value)
}

fn bind_styling_statement(
    mut styling_statement: parser::StylingStatement,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let type_ = styling_statement.type_.text(&context.loaded_files);
    let type_ = match type_ {
        "Label" | "Slide" | "Image" => StylingType::try_from(type_).unwrap(),
        _ => {
            context
                .diagnostics
                .report_unexpected_styling_type(type_, styling_statement.type_.location);
            return BoundNode::error(styling_statement.type_.location);
        }
    };

    binder.create_scope();
    let background = context.string_interner.create_or_get_variable("background");
    binder
        .expect_register_variable_id(
            background,
            Type::Background,
            styling_statement.name.location,
            context,
        )
        .unwrap();

    match type_ {
        StylingType::Label => {
            let text_color = context.string_interner.create_or_get_variable("text_color");

            binder
                .expect_register_variable_id(
                    text_color,
                    Type::Color,
                    styling_statement.name.location,
                    context,
                )
                .unwrap();
        }
        StylingType::Image => {
            let object_fit = context.string_interner.create_or_get_variable("object_fit");
            binder
                .expect_register_variable_id(
                    object_fit,
                    Type::ObjectFit,
                    styling_statement.name.location,
                    context,
                )
                .unwrap();
        }
        StylingType::Slide => {}
    }

    let mut body = Vec::with_capacity(styling_statement.body.len());
    for statement in styling_statement.body {
        body.push(bind_node(statement, binder, context));
    }

    styling_statement.body = Vec::new();

    binder.drop_scope();

    // Bind name last to check the body for errors!
    let Some(name) = binder.expect_register_variable_token(
        styling_statement.name,
        Type::Styling,
        styling_statement.name.location,
        context,
    ) else {
        return BoundNode::error(styling_statement.name.location);
    };
    BoundNode::styling_statement(styling_statement, location, name, type_, body)
}
