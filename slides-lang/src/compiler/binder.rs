use std::{collections::HashMap, path::PathBuf};

use convert_case::Casing;
use slides_rs_core::{CustomElement, ElementRefMut, Presentation, StyleUnit, TextAlign};
use string_interner::symbol::SymbolUsize;
use summum_types::summum;
use typing::{FunctionType, Type, TypeId, TypeInterner};

pub mod globals;
pub mod typing;

use super::{
    evaluator::{self, Value},
    lexer::Token,
    parser::{self, SyntaxNode, SyntaxNodeKind, debug_ast},
};
use crate::{Context, Location, StringInterner, VariableId};

pub(crate) fn create_presentation_from_file(file: PathBuf) -> slides_rs_core::Result<Presentation> {
    let mut context = Context::new();
    let file = context.load_file(file)?;
    let ast = parser::parse_file(file, &mut context);
    debug_ast(&ast, &context);
    let ast = bind_ast(ast, &mut context);
    debug_bound_ast(&ast, &context);
    // let Context {
    //     presentation,
    //     diagnostics,
    //     loaded_files,
    //     ..
    // } = context;
    if !context.diagnostics.is_empty() {
        context
            .diagnostics
            .write(&mut std::io::stdout(), &context.loaded_files)?;
    } else {
        evaluator::create_presentation_from_ast(ast, &mut context)?;
    }
    Ok(context.presentation)
}

fn debug_bound_ast(ast: &BoundAst, context: &Context) {
    for statement in &ast.statements {
        debug_bound_node(statement, context, String::new());
    }
}

fn debug_bound_node(statement: &BoundNode, context: &Context, indent: String) {
    print!("{indent}");
    match &statement.kind {
        BoundNodeKind::Error(()) => println!("#Error"),
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
        BoundNodeKind::ElementStatement(element_statement) => {
            println!(
                "CustomElement {} for {:?}",
                context
                    .string_interner
                    .resolve_variable(element_statement.name),
                context
                    .type_interner
                    .resolve(element_statement.type_)
                    .unwrap()
            );
            for statement in &element_statement.body {
                debug_bound_node(statement, context, format!("{indent}    "));
            }
        }
        BoundNodeKind::ImportStatement(path) => {
            println!("Import {}", path.display());
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
            println!(
                "FunctionCall: {:?}",
                context.type_interner.resolve(statement.type_).unwrap()
            );
            debug_bound_node(&function_call.base, context, format!("{indent}    "));
            for arg in &function_call.arguments {
                debug_bound_node(arg, context, format!("{indent}        "));
            }
        }
        BoundNodeKind::VariableReference(variable) => {
            println!(
                "Variable {}: {:?}",
                context.string_interner.resolve_variable(variable.id),
                context.type_interner.resolve(variable.type_).unwrap()
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
                "Variable Declaration {}: {:?}",
                context
                    .string_interner
                    .resolve_variable(variable_declaration.variable),
                context
                    .type_interner
                    .resolve(variable_declaration.value.type_)
                    .unwrap()
            );
            debug_bound_node(
                &variable_declaration.value,
                context,
                format!("{indent}    ="),
            );
        }
        BoundNodeKind::Dict(items) => {
            println!("Dict:");
            for (name, entry) in items {
                debug_bound_node(entry, context, format!("{indent}    {name}: "));
            }
        }
        BoundNodeKind::Array(items) => {
            println!("Array:");
            for entry in items {
                debug_bound_node(entry, context, format!("{indent}    "));
            }
        }
        BoundNodeKind::MemberAccess(member_access) => {
            println!(
                "Member Access .{}",
                context.string_interner.resolve(member_access.member)
            );
            debug_bound_node(&member_access.base, context, format!("{indent}    "));
        }
        BoundNodeKind::Conversion(conversion) => {
            println!(
                "Conversion to {:?} (Kind {:?})",
                context.type_interner.resolve(statement.type_).unwrap(),
                conversion.kind
            );
            debug_bound_node(&conversion.base, context, format!("{indent}    "));
        }
        BoundNodeKind::PostInitialization(post_initialization) => {
            println!("Post Initialization");
            debug_bound_node(&post_initialization.base, context, format!("{indent}    "));
            debug_bound_node(&post_initialization.dict, context, format!("{indent}    "));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub id: VariableId,
    pub definition: Location,
    pub type_: TypeId,
}

struct Scope {
    variables: HashMap<VariableId, Variable>,
}

impl Scope {
    pub fn global(string_interner: &mut StringInterner, type_interner: &mut TypeInterner) -> Self {
        let mut global = Self {
            variables: HashMap::new(),
        };
        let f = globals::FUNCTIONS;
        for function in f {
            let id = string_interner.create_or_get_variable(function.name);
            let argument_types = function
                .parameters
                .into_iter()
                .map(|t| type_interner.get_or_intern(t.clone()))
                .collect();
            let return_type = type_interner.get_or_intern(function.return_type.clone());
            let type_ = type_interner.get_or_intern(Type::Function(FunctionType {
                argument_types,
                return_type,
            }));
            global
                .try_register_variable(id, type_, Location::zero())
                .expect("infallible");
        }

        for enum_ in globals::ENUMS {
            let id = string_interner.create_or_get_variable(enum_.name);
            let type_ = type_interner.get_or_intern(Type::Enum(
                Box::new(enum_.type_),
                enum_
                    .variants
                    .into_iter()
                    .copied()
                    .map(Into::into)
                    .collect(),
            ));
            global
                .try_register_variable(id, type_, Location::zero())
                .expect("infallible");
        }

        debug_scope("globals", &global, string_interner);

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
        type_: TypeId,
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

fn debug_scope(name: &str, scope: &Scope, interner: &StringInterner) {
    println!("Scope {name}");
    println!();
    for (id, variable) in &scope.variables {
        let name = interner.resolve_variable(*id);
        println!("Variable {name}: {:?}", variable.type_);
    }
}

struct Binder {
    scopes: Vec<Scope>,
    types: HashMap<SymbolUsize, TypeId>,
}

impl Binder {
    pub fn new(interner: &mut StringInterner, type_interner: &mut TypeInterner) -> Self {
        Self {
            scopes: vec![Scope::global(interner, type_interner)],
            types: Type::simple_types()
                .into_iter()
                .map(|t| {
                    (
                        interner.create_or_get(t.as_ref()),
                        type_interner.get_or_intern(t),
                    )
                })
                .collect(),
        }
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("There is at least one scope")
    }

    fn expect_register_variable_token(
        &mut self,
        token: Token,
        type_: TypeId,
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
        type_: TypeId,
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

    fn look_up_type_by_name(&self, type_name: SymbolUsize) -> Option<TypeId> {
        self.types.get(&type_name).copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConversionKind {
    Implicit,
    TypedString,
}

#[derive(Debug, strum::EnumString, Clone, Copy, PartialEq, Eq)]
pub enum StylingType {
    Label,
    Image,
    Slide,
}

#[derive(Debug, Clone)]
pub struct StylingStatement {
    pub name: VariableId,
    pub type_: StylingType,
    pub body: Vec<BoundNode>,
}

#[derive(Debug, Clone)]
pub struct AssignmentStatement {
    pub lhs: Box<BoundNode>,
    pub value: Box<BoundNode>,
}

#[derive(Debug, Clone)]
pub struct ElementStatement {
    pub name: VariableId,
    pub type_: TypeId,
    pub parameters: Vec<VariableId>,
    pub body: Vec<BoundNode>,
}
#[derive(Debug, Clone)]

pub struct FunctionCall {
    pub base: Box<BoundNode>,
    pub arguments: Vec<BoundNode>,
    pub function_type: FunctionType,
}
#[derive(Debug, Clone)]

pub struct SlideStatement {
    pub name: VariableId,
    pub body: Vec<BoundNode>,
}
#[derive(Debug, Clone)]

pub struct VariableDeclaration {
    pub variable: VariableId,
    pub value: Box<BoundNode>,
}
#[derive(Debug, Clone)]

pub struct MemberAccess {
    pub base: Box<BoundNode>,
    pub member: SymbolUsize,
}

#[derive(Debug, Clone)]

pub struct Conversion {
    pub base: Box<BoundNode>,
    pub kind: ConversionKind,
    pub target: TypeId,
}

#[derive(Debug, Clone)]
pub struct PostInitialization {
    pub base: Box<BoundNode>,
    pub dict: Box<BoundNode>,
}

summum! {

#[derive(Debug, Clone)]
pub enum BoundNodeKind {
    Error(()),
    StylingStatement(StylingStatement),
    AssignmentStatement(AssignmentStatement),
    ElementStatement(ElementStatement),
    ImportStatement(PathBuf),
    FunctionCall(FunctionCall),
    VariableReference(Variable),
    Literal(Value),
    SlideStatement(SlideStatement),
    VariableDeclaration(VariableDeclaration),
    Dict(Vec<(String, BoundNode)>),
    Array(Vec<BoundNode>),
    MemberAccess(MemberAccess),
    Conversion(Conversion),
    PostInitialization(PostInitialization),
}
}
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BoundNode {
    base: Option<SyntaxNodeKind>,
    location: Location,
    pub kind: BoundNodeKind,
    pub type_: TypeId,
}
impl BoundNode {
    fn syntax_error(location: Location, consumed: bool) -> BoundNode {
        BoundNode {
            base: Some(SyntaxNodeKind::Error(consumed)),
            location,
            kind: BoundNodeKind::Error(()),
            type_: TypeId::ERROR,
        }
    }

    fn error(location: Location) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::Error(()),
            type_: TypeId::ERROR,
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
            type_: TypeId::VOID,
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
            type_: TypeId::VOID,
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

    fn literal(token: super::lexer::Token, value: Value, type_: TypeId) -> BoundNode {
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
            type_: TypeId::VOID,
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
            type_: TypeId::VOID,
        }
    }

    fn dict(
        dict: parser::Dict,
        location: Location,
        entries: Vec<(String, BoundNode)>,
        type_: TypeId,
    ) -> BoundNode {
        BoundNode {
            base: Some(SyntaxNodeKind::Dict(dict)),
            location,
            kind: BoundNodeKind::Dict(entries),
            type_,
        }
    }

    fn member_access(
        location: Location,
        base: BoundNode,
        member: SymbolUsize,
        type_: TypeId,
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

    fn conversion(base: BoundNode, target: TypeId, kind: ConversionKind) -> BoundNode {
        BoundNode {
            base: None,
            location: base.location,
            kind: BoundNodeKind::Conversion(Conversion {
                base: Box::new(base),
                kind,
                target: target,
            }),
            type_: target,
        }
    }

    fn post_initialization(location: Location, base: BoundNode, dict: BoundNode) -> BoundNode {
        BoundNode {
            base: None,
            location,
            type_: base.type_,
            kind: BoundNodeKind::PostInitialization(PostInitialization {
                base: Box::new(base),
                dict: Box::new(dict),
            }),
        }
    }

    fn element_statement(
        location: Location,
        element_type: TypeId,
        parameters: Vec<VariableId>,
        function_type: TypeId,
        name: VariableId,
        body: Vec<BoundNode>,
    ) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::ElementStatement(ElementStatement {
                type_: element_type,
                parameters,
                name,
                body,
            }),
            type_: function_type,
        }
    }

    fn import(path: PathBuf, location: Location) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::ImportStatement(path),
            type_: TypeId::VOID,
        }
    }

    fn array(entries: Vec<BoundNode>, location: Location, type_: TypeId) -> BoundNode {
        BoundNode {
            base: None,
            location,
            kind: BoundNodeKind::Array(entries),
            type_,
        }
    }
}

pub struct BoundAst {
    pub statements: Vec<BoundNode>,
}

fn bind_ast(ast: parser::Ast, context: &mut Context) -> BoundAst {
    let mut binder = Binder::new(&mut context.string_interner, &mut context.type_interner);
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
        SyntaxNodeKind::ElementStatement(element_statement) => {
            bind_element_statement(element_statement, statement.location, binder, context)
        }
        SyntaxNodeKind::ImportStatement(import_statement) => {
            bind_import_statement(import_statement, statement.location, binder, context)
        }
        SyntaxNodeKind::ExpressionStatement(expression_statement) => {
            let mut result = bind_node(*expression_statement.expression, binder, context);
            result.type_ = TypeId::VOID;
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
        SyntaxNodeKind::Error(consumed) => BoundNode::syntax_error(statement.location, consumed),
        SyntaxNodeKind::Dict(dict) => bind_dict(dict, statement.location, binder, context),
        SyntaxNodeKind::Array(array) => bind_array(array, statement.location, binder, context),
        SyntaxNodeKind::PostInitialization(post_initialization) => {
            bind_post_initialization(post_initialization, statement.location, binder, context)
        }
        unsupported => unreachable!("Not supported: {}", unsupported.as_ref()),
    }
}

fn bind_array(
    array: parser::Array,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let mut entries = Vec::with_capacity(array.entries.len());
    let mut inner_type = TypeId::ERROR;
    for (entry, _) in array.entries {
        let entry = bind_node(entry, binder, context);
        if inner_type == TypeId::ERROR {
            inner_type = entry.type_;
        }
        entries.push(entry);
    }
    if inner_type == TypeId::ERROR {
        // TODO: Array is under specified
        BoundNode::error(location)
    } else {
        let type_ = context.type_interner.get_or_intern(Type::Array(inner_type));
        BoundNode::array(entries, location, type_)
    }
}

fn bind_import_statement(
    import_statement: parser::ImportStatement,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let path = bind_node(*import_statement.path, binder, context);
    if path.type_ != TypeId::PATH {
        // TODO: Argument must be path!
        return BoundNode::error(location);
    }
    let Ok(path) = path.kind.try_into_conversion() else {
        // TODO: Argument must be literal!
        return BoundNode::error(location);
    };
    let Ok(literal) = path.base.kind.try_into_literal() else {
        // TODO: Argument must be literal!
        return BoundNode::error(location);
    };
    let Ok(string) = literal.try_into_string() else {
        // TODO: Argument must be string! Should never get to here probably!
        return BoundNode::error(location);
    };
    let path = PathBuf::from(string);
    if !path.exists() {
        // TODO: Path must be existing at compile time!
        return BoundNode::error(location);
    }
    BoundNode::import(path, location)
}

fn bind_element_statement(
    element_statement: parser::ElementStatement,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let type_name = element_statement
        .name
        .text(&context.loaded_files)
        .to_case(convert_case::Case::Pascal);
    let element_type = context
        .type_interner
        .get_or_intern(Type::CustomElement(type_name));

    let scope = binder.create_scope();
    for (name, type_) in globals::find_members_by_name("Element") {
        let id = context.string_interner.create_or_get_variable(name);
        let type_ = context.type_interner.get_or_intern(type_);
        scope
            .try_register_variable(id, type_, element_statement.name.location)
            .expect("cannot fail");
    }
    let parameters = bind_parameter_block(
        element_statement
            .parameters
            .kind
            .try_as_parameter_block()
            .expect("Parameters should be parameters"),
        element_statement.parameters.location,
        binder,
        context,
    );
    let function_type = Type::Function(FunctionType {
        argument_types: parameters
            .iter()
            .map(|v| binder.look_up_variable(*v).unwrap().type_)
            .collect(),
        return_type: element_type,
    });
    let function_type = context.type_interner.get_or_intern(function_type);
    let mut body = Vec::with_capacity(element_statement.body.len());
    for statement in element_statement.body {
        body.push(bind_node(statement, binder, context));
    }
    binder.drop_scope();
    let Some(name) = binder.expect_register_variable_token(
        element_statement.name,
        function_type,
        element_statement.name.location,
        context,
    ) else {
        return BoundNode::error(element_statement.name.location);
    };
    BoundNode::element_statement(
        location,
        element_type,
        parameters,
        function_type,
        name,
        body,
    )
}

fn bind_parameter_block(
    parameter_block: parser::ParameterBlock,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> Vec<VariableId> {
    let mut result = Vec::with_capacity(parameter_block.parameters.len());
    for (parameter, _) in parameter_block.parameters {
        let location = parameter.location;
        let parameter = parameter
            .kind
            .try_as_parameter()
            .expect("Parameter blocks are made of parameter");
        let type_name_str = parameter.type_.text(&context.loaded_files);
        let type_name = context.string_interner.create_or_get(type_name_str);
        let type_ = match binder.look_up_type_by_name(type_name) {
            Some(type_) => type_,
            None => {
                // TODO: Diagnostics
                panic!("Found no type named {type_name_str}");
                TypeId::ERROR
            }
        };
        let variable = context
            .string_interner
            .create_or_get_variable(parameter.identifier.text(&context.loaded_files));
        let variable = match binder.expect_register_variable_id(variable, type_, location, context)
        {
            Some(it) => it,
            None => variable,
        };
        result.push(variable);
    }
    result
}

fn bind_post_initialization(
    post_initialization: parser::PostInitialization,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let base = bind_node(*post_initialization.expression, binder, context);
    let mut dict = bind_node(*post_initialization.dict, binder, context);
    for (entry, entry_type) in dict.kind.as_mut_dict() {
        let member = context.string_interner.create_or_get(&entry);
        let base_type = context
            .type_interner
            .resolve(base.type_)
            .unwrap_or(&Type::Error)
            .clone();
        let mut base = base.clone();
        if let Some(target) = access_member(
            entry_type.location,
            binder,
            context,
            // TODO: This is iffy, but it is also very much not clear what
            // should happen here!
            &mut base,
            member,
            base_type,
        ) {
            let mut fallback = BoundNode::error(entry_type.location);
            std::mem::swap(entry_type, &mut fallback);
            *entry_type =
                bind_conversion(fallback, target, ConversionKind::Implicit, binder, context)
        } else {
            context.diagnostics.report_unknown_member(
                dict.location,
                &context
                    .type_interner
                    .resolve(base.type_)
                    .unwrap_or(&Type::Error),
                &entry,
            );
            *entry_type = BoundNode::error(entry_type.location);
        }
    }
    BoundNode::post_initialization(location, base, dict)
}

fn bind_member_access(
    member_access: parser::MemberAccess,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let mut base = bind_node(*member_access.base, binder, context);
    let member = member_access.member.text(&context.loaded_files);
    let member = context.string_interner.create_or_get(member);
    let base_type = context
        .type_interner
        .resolve(base.type_)
        .unwrap_or(&Type::Error)
        .clone();
    let Some(member_type) = access_member(
        member_access.member.location,
        binder,
        context,
        &mut base,
        member,
        base_type,
    ) else {
        return BoundNode::error(location);
    };
    BoundNode::member_access(location, base, member, member_type)
}

fn access_member(
    error_location: Location,
    binder: &mut Binder,
    context: &mut Context,
    base: &mut BoundNode,
    member: SymbolUsize,
    base_type: Type,
) -> Option<TypeId> {
    let mut types_to_check = vec![base_type];
    let mut visited = Vec::new();
    while let Some(base_type) = types_to_check.pop() {
        if visited.contains(&base_type) {
            continue;
        } else {
            visited.push(base_type.clone());
        }
        let member = context.string_interner.resolve(member);
        dbg!(&base_type, member);
        if let Some(type_) = base_type.field_type(member) {
            let type_ = context.type_interner.get_or_intern(type_);
            let mut fallback = BoundNode::error(base.location);
            std::mem::swap(base, &mut fallback);
            *base = bind_conversion(
                fallback,
                context.type_interner.get_or_intern(base_type),
                ConversionKind::Implicit,
                binder,
                context,
            );
            return Some(type_);
        }
        types_to_check
            .extend_from_slice(base_type.get_available_conversions(ConversionKind::Implicit));
    }
    context.diagnostics.report_unknown_member(
        error_location,
        &context
            .type_interner
            .resolve(base.type_)
            .unwrap_or(&Type::Error),
        context.string_interner.resolve(member),
    );
    None
}

fn bind_dict(
    mut dict: parser::Dict,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let mut entries = Vec::with_capacity(dict.entries.len());
    for (entry, _) in dict.entries {
        let Some(entry) = entry.kind.try_as_dict_entry() else {
            continue;
        };
        let key = entry.identifier.text(&context.loaded_files).to_string();
        let value = bind_node(*entry.value, binder, context);
        entries.push((key, value));
    }
    dict.entries = Vec::new();
    let types: Vec<Variable> = entries
        .iter()
        .map(|(n, b)| {
            let variable_id = context.string_interner.create_or_get_variable(&n);
            Variable {
                id: variable_id,
                definition: b.location,
                type_: b.type_,
            }
        })
        .collect();
    let type_ = context.type_interner.get_or_intern(Type::TypedDict(types));
    BoundNode::dict(dict, location, entries, type_)
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
    let scope = binder.create_scope();
    for (name, type_) in globals::find_members_by_name("Slide") {
        let id = context.string_interner.create_or_get_variable(name);
        let type_ = context.type_interner.get_or_intern(type_);
        scope
            .try_register_variable(id, type_, slide_statement.name.location)
            .expect("infallible");
    }
    let mut statements = Vec::with_capacity(slide_statement.body.len());
    for statement in slide_statement.body {
        statements.push(bind_node(statement, binder, context));
    }
    binder.drop_scope();
    slide_statement.body = Vec::new();
    let type_ = context.type_interner.get_or_intern(Type::Slide);
    let Some(name) = binder.expect_register_variable_token(
        slide_statement.name,
        type_,
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
    let text = typed_string.string.text(&context.loaded_files);
    let value = Value::parse_string_literal(text, true);
    let type_ = context.type_interner.get_or_intern(value.infer_type());
    let literal = BoundNode::literal(typed_string.string, value, type_);
    let type_ = typed_string.type_.text(&context.loaded_files);
    let type_ = match type_ {
        "c" => Type::Color,
        "l" => Type::Label,
        "p" => Type::Path,
        unknown => {
            context
                .diagnostics
                .report_unknown_string_type(unknown, location);
            return BoundNode::error(typed_string.type_.location);
        }
    };
    let type_ = context.type_interner.get_or_intern(type_);
    bind_conversion(literal, type_, ConversionKind::TypedString, binder, context)
}

fn bind_conversion(
    base: BoundNode,
    target: TypeId,
    conversion_kind: ConversionKind,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    if base.type_ == TypeId::ERROR || base.type_ == target {
        return base;
    }
    let style_unit_type = context.type_interner.get_or_intern(Type::StyleUnit);
    match conversion_kind {
        ConversionKind::Implicit => match context.type_interner.resolve_types([base.type_, target])
        {
            [from @ Type::TypedDict(fields), Type::Thickness] => {
                for field in fields {
                    if field.type_ != style_unit_type {
                        context.diagnostics.report_cannot_convert(
                            context.type_interner.resolve(field.type_).unwrap(),
                            &Type::StyleUnit,
                            field.definition,
                        );
                        return BoundNode::error(field.definition);
                    }
                    if !["top", "bottom", "left", "right"]
                        .contains(&context.string_interner.resolve_variable(field.id))
                    {
                        context.diagnostics.report_cannot_convert(
                            from,
                            &Type::StyleUnit,
                            field.definition,
                        );
                        return BoundNode::error(field.definition);
                    }
                }
            }
            [Type::Color, Type::Background] => {}
            [
                Type::Label | Type::Image | Type::CustomElement(_),
                Type::Element,
            ] => {}
            [Type::Error, _] => {
                return BoundNode::error(base.location);
            }
            [_, Type::Error] => {
                return BoundNode::error(base.location);
            }
            [from, to] => {
                context
                    .diagnostics
                    .report_cannot_convert(from, to, base.location);
                return BoundNode::error(base.location);
            }
        },
        ConversionKind::TypedString => match context.type_interner.resolve(target) {
            Some(Type::Label | Type::Color | Type::Path) => {}
            unknown => unreachable!("Unknown TypedString {unknown:?}"),
        },
    }
    BoundNode::conversion(base, target, conversion_kind)
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
        super::lexer::TokenKind::String => Value::parse_string_literal(text, true),
        super::lexer::TokenKind::StyleUnitLiteral => {
            Value::StyleUnit(text.parse().expect("lexer filtered"))
        }
        err => unreachable!("This is a unhandled literal {err:?}"),
    };
    let type_ = context.type_interner.get_or_intern(value.infer_type());
    BoundNode::literal(token, value, type_)
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
    let Some(function_type) = context
        .type_interner
        .resolve(base.type_)
        .unwrap_or(&Type::Error)
        .clone()
        .try_as_function()
    else {
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
    let lhs = bind_node(*assignment_statement.lhs, binder, context);
    let value = bind_node(*assignment_statement.assignment, binder, context);
    let value = bind_conversion(value, lhs.type_, ConversionKind::Implicit, binder, context);
    BoundNode::assignment_statement(location, lhs, value)
}

fn bind_styling_statement(
    mut styling_statement: parser::StylingStatement,
    location: Location,
    binder: &mut Binder,
    context: &mut Context,
) -> BoundNode {
    let type_ = styling_statement.type_.text(&context.loaded_files);
    let (type_, members) = match type_ {
        "Label" | "Slide" | "Image" => (
            StylingType::try_from(type_).unwrap(),
            globals::find_members_by_name(type_),
        ),
        _ => {
            context
                .diagnostics
                .report_unexpected_styling_type(type_, styling_statement.type_.location);
            return BoundNode::error(styling_statement.type_.location);
        }
    };

    binder.create_scope();

    for (member_name, member_type) in members {
        dbg!(member_name, &member_type);
        let variable = context.string_interner.create_or_get_variable(&member_name);
        let type_id = context.type_interner.get_or_intern(member_type);
        binder
            .expect_register_variable_id(
                variable,
                type_id,
                styling_statement.name.location,
                context,
            )
            .unwrap();
    }

    let mut body = Vec::with_capacity(styling_statement.body.len());
    for statement in styling_statement.body {
        body.push(bind_node(statement, binder, context));
    }

    styling_statement.body = Vec::new();

    binder.drop_scope();

    // Bind name last to check the body for errors!
    let styling_type = context.type_interner.get_or_intern(Type::Styling);
    let name = &context.loaded_files[styling_statement.name.location];

    let Some(name) = (if name == "default" {
        Some(context.string_interner.create_or_get_variable("default"))
    } else {
        binder.expect_register_variable_token(
            styling_statement.name,
            styling_type,
            styling_statement.name.location,
            context,
        )
    }) else {
        return BoundNode::error(styling_statement.name.location);
    };
    BoundNode::styling_statement(styling_statement, location, name, type_, body)
}
