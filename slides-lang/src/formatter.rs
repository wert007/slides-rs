use std::{
    fs::File,
    io::{Result, Write},
};

use crate::{
    Context,
    compiler::{self, lexer::Token, parser::SyntaxNodeKind},
};

struct Formatter<W: std::io::Write> {
    indent: usize,
    w: W,
    last_written_byte: u8,
    column: usize,
}

impl<W: Write> Formatter<W> {
    fn new(w: W) -> Self {
        Self {
            indent: 0,
            w,
            last_written_byte: 0,
            column: 0,
        }
    }

    fn emit_indent(&mut self) -> Result<()> {
        if self.column < self.indent {
            dbg!(self.column, self.indent);
            let buffer = vec![b' '; self.indent - self.column];
            self.w.write(&buffer)?;
            self.column = self.indent;
        }
        Ok(())
    }

    fn ensure_empty_line(&mut self) -> Result<()> {
        writeln!(self.w)?;
        self.column = 0;
        Ok(())
    }

    fn ensure_indented_line(&mut self) -> Result<()> {
        if self.last_written_byte != b'\n' {
            self.ensure_empty_line()?;
        }
        self.emit_indent()?;
        Ok(())
    }
}

impl<W: Write> Write for Formatter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let str = std::str::from_utf8(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        self.emit_indent()?;
        for (i, line) in str.lines().enumerate() {
            if i > 0 {
                self.w.write(&[b'\n'])?;
                self.column = 0;
                self.emit_indent()?;
            }
            self.w.write(line.as_bytes())?;
            // TODO: Utf-8
            self.column += line.len();
        }
        self.last_written_byte = buf[buf.len() - 1];
        if self.last_written_byte == b'\n' {
            self.w.write(&[b'\n'])?;
            self.column = 0;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        self.w.flush()
    }
}

pub fn format_file(file: std::path::PathBuf) -> std::io::Result<()> {
    let mut context = Context::new();
    let mut formatter = Formatter::new(File::create("out.sld")?);
    let file = context.load_file(file)?;
    let ast = compiler::parser::parse_file(file, &mut context);
    format_ast(ast, &mut formatter, &mut context)?;
    Ok(())
}

fn format_ast<W: Write>(
    ast: compiler::parser::Ast,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::io::Result<()> {
    for statement in ast.statements {
        format_node(statement, formatter, context)?;
    }
    formatter.ensure_empty_line()?;
    Ok(())
}

fn format_node<W: Write>(
    node: compiler::parser::SyntaxNode,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::io::Result<()> {
    match node.kind {
        SyntaxNodeKind::StylingStatement(styling_statement) => {
            format_styling_statement(styling_statement, formatter, context)
        }
        SyntaxNodeKind::ExpressionStatement(expression_statement) => todo!(),
        SyntaxNodeKind::VariableDeclaration(variable_declaration) => {
            format_variable_declaration(variable_declaration, formatter, context)
        }
        SyntaxNodeKind::SlideStatement(slide_statement) => {
            format_slide_statement(slide_statement, formatter, context)
        }
        SyntaxNodeKind::Literal(token) | SyntaxNodeKind::VariableReference(token) => {
            emit_token(token, formatter, context)
        }
        SyntaxNodeKind::MemberAccess(member_access) => {
            format_member_access(member_access, formatter, context)
        }
        SyntaxNodeKind::AssignmentStatement(assignment_statement) => {
            format_assignment_statement(assignment_statement, formatter, context)
        }
        SyntaxNodeKind::FunctionCall(function_call) => {
            format_function_call(function_call, formatter, context)
        }
        SyntaxNodeKind::TypedString(typed_string) => {
            format_typed_string(typed_string, formatter, context)
        }
        SyntaxNodeKind::Error => todo!(),
        SyntaxNodeKind::DictEntry(dict_entry) => format_dict_entry(dict_entry, formatter, context),
        SyntaxNodeKind::Dict(dict) => format_dict(dict, formatter, context),
        SyntaxNodeKind::InferredMember(inferred_member) => todo!(),
        SyntaxNodeKind::PostInitialization(post_initialization) => {
            format_post_initialization(post_initialization, formatter, context)
        }
    }
}

fn format_dict_entry<W: Write>(
    dict_entry: compiler::parser::DictEntry,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::result::Result<(), std::io::Error> {
    formatter.emit_indent()?;
    write!(
        formatter,
        "{}: ",
        dict_entry.identifier.text(&context.loaded_files)
    )?;
    format_node(*dict_entry.value, formatter, context)?;
    Ok(())
}

fn format_dict<W: Write>(
    dict: compiler::parser::Dict,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    writeln!(formatter, "{{")?;
    formatter.indent += 4;
    for (entry, _) in dict.entries {
        format_node(entry, formatter, context)?;
        writeln!(formatter, ",")?;
    }
    formatter.indent -= 4;
    formatter.flush()?;
    write!(formatter, "}}")?;
    formatter.flush()?;
    Ok(())
}

fn format_post_initialization<W: Write>(
    post_initialization: compiler::parser::PostInitialization,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*post_initialization.expression, formatter, context)?;
    write!(formatter, " ")?;
    format_node(*post_initialization.dict, formatter, context)?;
    Ok(())
}

fn format_member_access<W: Write>(
    member_access: compiler::parser::MemberAccess,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*member_access.base, formatter, context)?;
    write!(
        formatter,
        ".{}",
        member_access.member.text(&context.loaded_files)
    )?;
    Ok(())
}

fn format_variable_declaration<W: Write>(
    variable_declaration: compiler::parser::VariableDeclaration,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.ensure_indented_line()?;
    write!(
        formatter,
        "let {} = ",
        variable_declaration.name.text(&context.loaded_files)
    )?;
    format_node(*variable_declaration.expression, formatter, context)?;
    writeln!(formatter, ";")?;
    Ok(())
}

fn format_slide_statement<W: Write>(
    slide_statement: compiler::parser::SlideStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    writeln!(
        formatter,
        "slide {}:",
        slide_statement.name.text(&context.loaded_files)
    )?;
    formatter.indent += 4;
    for statement in slide_statement.body {
        format_node(statement, formatter, context)?;
    }
    formatter.indent -= 4;
    formatter.ensure_empty_line()?;
    Ok(())
}

fn format_typed_string<W: Write>(
    typed_string: compiler::parser::TypedString,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    emit_token(typed_string.type_, formatter, context)?;
    emit_token(typed_string.string, formatter, context)?;
    Ok(())
}

fn format_function_call<W: Write>(
    function_call: compiler::parser::FunctionCall,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*function_call.base, formatter, context)?;
    write!(formatter, "(")?;
    for (i, (argument, _)) in function_call.arguments.into_iter().enumerate() {
        if i > 0 {
            write!(formatter, ", ")?;
        }
        format_node(argument, formatter, context)?;
    }
    write!(formatter, ")")?;
    Ok(())
}

fn emit_token<W: Write>(
    token: Token,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::result::Result<(), std::io::Error> {
    let indent = formatter.indent;
    formatter.indent = 0;
    write!(formatter, "{}", token.text(&context.loaded_files))?;
    formatter.indent = indent;
    Ok(())
}

fn format_assignment_statement<W: Write>(
    assignment_statement: compiler::parser::AssignmentStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.ensure_indented_line()?;
    format_node(*assignment_statement.lhs, formatter, context)?;
    write!(formatter, " = ")?;
    format_node(*assignment_statement.assignment, formatter, context)?;
    writeln!(formatter, ";")?;
    Ok(())
}

fn format_styling_statement<W: std::io::Write>(
    styling_statement: compiler::parser::StylingStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    let name = styling_statement.name.text(&context.loaded_files);
    let type_ = styling_statement.type_.text(&context.loaded_files);
    writeln!(formatter, "styling {name}({type_}):")?;
    formatter.indent += 4;
    for statement in styling_statement.body {
        format_node(statement, formatter, context)?;
    }
    formatter.indent -= 4;
    formatter.ensure_empty_line()?;
    Ok(())
}
