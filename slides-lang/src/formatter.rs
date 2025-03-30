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
    trim_lines: bool,
}

impl<W: Write> Formatter<W> {
    fn new(w: W) -> Self {
        Self {
            indent: 0,
            w,
            last_written_byte: 0,
            column: 0,
            trim_lines: false,
        }
    }

    fn ensure_indent(&mut self) -> Result<()> {
        if self.column < self.indent {
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
        self.ensure_indent()?;
        Ok(())
    }

    fn emit_token(
        &mut self,
        token: Token,
        files: &crate::Files,
        insert_trailing_space: bool,
    ) -> std::io::Result<()> {
        if let Some(leading) = token.trivia.leading_comments {
            self.ensure_indented_line()?;
            self.trim_lines = true;
            write!(self, "{}", &files[leading])?;
            // self.ensure_indented_line()?;
            self.trim_lines = false;
        }
        write!(self, "{}", &files[token.location])?;
        if insert_trailing_space {
            self.write(&[b' '])?;
        }
        if let Some(trailing) = token.trivia.trailing_comments {
            // TODO: self.reserve(trailing.length);
            if !insert_trailing_space {
                self.write(&[b' '])?;
            }
            let indent = self.indent;
            self.indent = self.column;
            self.trim_lines = true;
            writeln!(self, "{}", &files[trailing])?;
            self.trim_lines = false;
            self.indent = indent;
        }
        Ok(())
    }

    fn ensure_space(&mut self) -> std::io::Result<()> {
        if self.last_written_byte != b' ' {
            self.write(&[b' '])?;
        }
        Ok(())
    }

    fn ensure_new_line(&mut self) -> std::io::Result<()> {
        if self.last_written_byte != b'\n' {
            self.write(&[b'\n'])?;
        }
        Ok(())
    }
}

impl<W: Write> Write for Formatter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let str = std::str::from_utf8(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        self.ensure_indent()?;
        for (i, line) in str.lines().enumerate() {
            if i > 0 {
                self.w.write(&[b'\n'])?;
                self.column = 0;
                self.ensure_indent()?;
            }
            let line = if self.trim_lines { line.trim() } else { line };
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
    formatter.emit_token(ast.eof, &context.loaded_files, false)?;
    formatter.ensure_new_line()?;
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
            formatter.emit_token(token, &context.loaded_files, false)
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
    formatter.ensure_indent()?;
    formatter.emit_token(dict_entry.identifier, &context.loaded_files, false)?;
    formatter.emit_token(dict_entry.colon, &context.loaded_files, true)?;
    format_node(*dict_entry.value, formatter, context)?;
    Ok(())
}

fn format_dict<W: Write>(
    dict: compiler::parser::Dict,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(dict.lbrace, &context.loaded_files, false)?;
    formatter.ensure_new_line()?;
    formatter.indent += 4;
    for (entry, comma) in dict.entries {
        format_node(entry, formatter, context)?;
        match comma {
            Some(it) => formatter.emit_token(it, &context.loaded_files, false)?,
            None => write!(formatter, ",")?,
        }
        formatter.ensure_new_line()?;
    }
    formatter.indent -= 4;
    formatter.emit_token(dict.rbrace, &context.loaded_files, false)?;
    Ok(())
}

fn format_post_initialization<W: Write>(
    post_initialization: compiler::parser::PostInitialization,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*post_initialization.expression, formatter, context)?;
    formatter.ensure_space()?;
    format_node(*post_initialization.dict, formatter, context)?;
    Ok(())
}

fn format_member_access<W: Write>(
    member_access: compiler::parser::MemberAccess,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*member_access.base, formatter, context)?;
    // formatter.reserve()
    formatter.emit_token(member_access.period, &context.loaded_files, false)?;
    formatter.emit_token(member_access.member, &context.loaded_files, false)?;
    Ok(())
}

fn format_variable_declaration<W: Write>(
    variable_declaration: compiler::parser::VariableDeclaration,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.ensure_indented_line()?;
    formatter.emit_token(
        variable_declaration.let_keyword,
        &context.loaded_files,
        true,
    )?;
    formatter.emit_token(variable_declaration.name, &context.loaded_files, true)?;
    formatter.emit_token(variable_declaration.equals, &context.loaded_files, true)?;
    // TODO: formatter.reserve(variable_declaration.expression.location.length)
    format_node(*variable_declaration.expression, formatter, context)?;
    formatter.emit_token(variable_declaration.semicolon, &context.loaded_files, false)?;
    formatter.ensure_new_line()?;
    Ok(())
}

fn format_slide_statement<W: Write>(
    slide_statement: compiler::parser::SlideStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(slide_statement.slide_keyword, &context.loaded_files, true)?;
    formatter.emit_token(slide_statement.name, &context.loaded_files, false)?;
    formatter.emit_token(slide_statement.colon, &context.loaded_files, false)?;
    formatter.ensure_new_line()?;
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
    // TODO: Make it illegal to write `p "path"`
    formatter.emit_token(typed_string.type_, &context.loaded_files, false)?;
    formatter.emit_token(typed_string.string, &context.loaded_files, false)?;
    Ok(())
}

fn format_function_call<W: Write>(
    function_call: compiler::parser::FunctionCall,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*function_call.base, formatter, context)?;
    formatter.emit_token(function_call.lparen, &context.loaded_files, false)?;
    let arguments_count = function_call.arguments.len();
    for (i, (argument, comma)) in function_call.arguments.into_iter().enumerate() {
        format_node(argument, formatter, context)?;
        if i != arguments_count {
            match comma {
                Some(it) => formatter.emit_token(it, &context.loaded_files, true)?,
                None => write!(formatter, ", ")?,
            }
        }
    }
    formatter.emit_token(function_call.rparen, &context.loaded_files, false)?;
    Ok(())
}

fn format_assignment_statement<W: Write>(
    assignment_statement: compiler::parser::AssignmentStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.ensure_indented_line()?;
    format_node(*assignment_statement.lhs, formatter, context)?;
    formatter.ensure_space()?;
    formatter.emit_token(assignment_statement.equals, &context.loaded_files, true)?;
    format_node(*assignment_statement.assignment, formatter, context)?;
    formatter.emit_token(assignment_statement.semicolon, &context.loaded_files, false)?;
    formatter.ensure_new_line()?;
    Ok(())
}

fn format_styling_statement<W: std::io::Write>(
    styling_statement: compiler::parser::StylingStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(
        styling_statement.styling_keyword,
        &context.loaded_files,
        true,
    )?;
    formatter.emit_token(styling_statement.name, &context.loaded_files, false)?;
    formatter.emit_token(styling_statement.lparen, &context.loaded_files, false)?;
    formatter.emit_token(styling_statement.type_, &context.loaded_files, false)?;
    formatter.emit_token(styling_statement.rparen, &context.loaded_files, false)?;
    formatter.emit_token(styling_statement.colon, &context.loaded_files, false)?;
    formatter.indent += 4;
    formatter.ensure_new_line()?;
    for statement in styling_statement.body {
        format_node(statement, formatter, context)?;
    }
    formatter.indent -= 4;
    formatter.ensure_empty_line()?;
    Ok(())
}

// fn format_leading_trivia<W: Write>(
//     trivia: &[compiler::lexer::Trivia],
//     formatter: &mut Formatter<W>,
//     context: &mut Context,
// ) -> Result<()> {
//     for trivia in trivia.into_iter().filter_map(|t| t.comments_before) {
//         writeln!(formatter, "{}", &context.loaded_files[trivia].trim())?;
//     }
//     formatter.ensure_indent()?;
//     Ok(())
// }

// fn format_following_trivia<W: Write>(
//     trivia: &[compiler::lexer::Trivia],
//     formatter: &mut Formatter<W>,
//     context: &mut Context,
// ) -> Result<()> {
//     let indent = formatter.indent;
//     formatter.indent = formatter.column;
//     for trivia in trivia.into_iter().filter_map(|t| t.comments_after) {
//         writeln!(formatter, "{}", &context.loaded_files[trivia].trim())?;
//     }
//     formatter.indent = indent;
//     Ok(())
// }
