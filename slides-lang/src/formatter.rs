use std::{
    fmt,
    fs::File,
    io::{Result, Write, stdout},
};

use crate::{
    Context, Location,
    compiler::{
        self, DebugLang,
        evaluator::Value,
        lexer::{Token, TokenKind},
        parser::{SyntaxNodeKind, debug_ast},
    },
};

#[derive(Debug, Clone, Copy, Default)]
struct TokenConfig {
    leading_blank_line: bool,
    trailing_space: bool,
    trim_lines: bool,
    no_indent: bool,
    indent_inner_lines: bool,
}

impl TokenConfig {
    pub const TRAILING_SPACE: TokenConfig = TokenConfig {
        trailing_space: true,
        leading_blank_line: false,
        trim_lines: false,
        no_indent: false,
        indent_inner_lines: false,
    };

    pub const LEADING_BLANK_LINE: TokenConfig = TokenConfig {
        leading_blank_line: true,
        trailing_space: false,
        trim_lines: false,
        no_indent: false,
        indent_inner_lines: false,
    };

    pub const TRIMMED: TokenConfig = TokenConfig {
        trim_lines: true,
        trailing_space: false,
        leading_blank_line: false,
        no_indent: true,
        indent_inner_lines: false,
    };
}

#[derive(Debug)]
struct Formatter<W: std::io::Write> {
    indent: usize,
    w: W,
    line_buffer: Vec<u8>,
    last_written_byte: u8,
    column: usize,
    trim_lines: bool,
    is_start_of_file: bool,
    wanted_column_width: usize,
}

impl<W: Write + fmt::Debug> Formatter<W> {
    fn new(w: W, wanted_column_width: usize) -> Self {
        Self {
            indent: 0,
            w,
            line_buffer: Vec::new(),
            last_written_byte: b'\n',
            column: 0,
            trim_lines: false,
            is_start_of_file: true,
            wanted_column_width,
        }
    }

    fn ensure_indent(&mut self) -> Result<()> {
        if self.column < self.indent {
            let buffer = vec![b' '; self.indent - self.column];
            self.line_buffer.extend_from_slice(&buffer);
            self.last_written_byte = b' ';
            // self.write(&buffer)?;
            self.column = self.indent;
        }
        Ok(())
    }

    fn ensure_new_line(&mut self) -> Result<()> {
        if self.last_written_byte == b' '
            && !self.line_buffer.is_empty()
            && self.line_buffer.iter().all(|b| b.is_ascii_whitespace())
        {
            self.line_buffer.clear();
            self.column = 0;
        } else if self.last_written_byte != b'\n' {
            self.line_buffer.push(b'\n');
            self.flush()?;
            self.column = 0;
        }
        self.last_written_byte = b'\n';
        assert_eq!(self.column, 0);
        Ok(())
    }

    fn ensure_indented_line(&mut self) -> Result<()> {
        if self.last_written_byte != b'\n' {
            self.ensure_new_line()?;
        }
        self.ensure_indent()?;
        Ok(())
    }

    fn emit_token(
        &mut self,
        token: Token,
        files: &crate::Files,
        conf: TokenConfig,
    ) -> std::io::Result<()> {
        let indent = self.indent;
        let trim_lines = self.trim_lines;
        if token.trivia.leading_blank_line || conf.leading_blank_line {
            self.ensure_blank_line()?;
        }
        if let Some(leading) = token.trivia.leading_comments {
            self.ensure_indented_line()?;
            self.trim_lines = true;
            write!(self, "{}", &files[leading].trim())?;
            self.trim_lines = trim_lines;
            self.ensure_new_line()?;
        }
        self.ensure_indent()?;
        if conf.no_indent {
            self.indent = 0;
        }

        let line_count = files[token.location].lines().count();
        let ends_with_new_line = files[token.location]
            .as_bytes()
            .last()
            .copied()
            .unwrap_or_default()
            == b'\n';
        for (i, line) in files[token.location].lines().enumerate() {
            if conf.indent_inner_lines && line_count > 2 {
                if i == 1 {
                    self.indent += 4;
                } else if i == line_count - 1 {
                    self.indent -= 4;
                }
            }
            if i > 0 {
                writeln!(self)?;
            }
            let line = if conf.trim_lines { line.trim() } else { line };
            self.ensure_indent()?;
            write!(self, "{line}")?;
        }
        if ends_with_new_line {
            writeln!(self)?;
        }

        if conf.no_indent {
            self.indent = indent;
        }
        if conf.trailing_space {
            self.write(b" ")?;
        }
        if let Some(trailing) = token.trivia.trailing_comments {
            // TODO: self.reserve(trailing.length);
            if !conf.trailing_space {
                self.write(b" ")?;
            }
            let indent = self.indent;
            self.indent = self.column;
            self.trim_lines = true;
            write!(self, "{}", &files[trailing].trim())?;
            self.trim_lines = trim_lines;
            self.ensure_new_line()?;
            self.indent = indent;
        }
        Ok(())
    }

    fn ensure_space(&mut self) -> std::io::Result<()> {
        if self.last_written_byte != b' ' {
            self.write(b" ")?;
        }
        Ok(())
    }

    fn ensure_blank_line(&mut self) -> std::io::Result<()> {
        if !self.is_start_of_file {
            self.ensure_new_line()?;
            self.write(b"\n")?;
        }
        Ok(())
    }

    fn available_space(&self) -> usize {
        self.wanted_column_width.saturating_sub(self.column)
    }

    fn raw(&mut self, location: crate::Location, loaded_files: &crate::Files) -> Result<()> {
        self.w.write_all(&loaded_files[location].as_bytes())
    }
}

impl<W: Write + fmt::Debug> Write for Formatter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let str = std::str::from_utf8(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        // self.ensure_indent()?;
        for (i, line) in str.lines().enumerate() {
            if i > 0 {
                let trunc = self
                    .line_buffer
                    .iter()
                    .enumerate()
                    .rev()
                    .skip_while(|(_, b)| b == &&b' ')
                    .next()
                    .map(|(i, _)| i + 1)
                    .unwrap_or_default();

                self.line_buffer.truncate(trunc);
                self.line_buffer.push(b'\n');
                self.flush()?;
                self.column = 0;
                self.ensure_indent()?;
            }
            let line = if self.trim_lines { line.trim() } else { line };
            self.line_buffer.extend_from_slice(line.as_bytes());
            // TODO: Utf-8
            self.column += line.len();
        }
        self.last_written_byte = buf[buf.len() - 1];
        if self.last_written_byte == b'\n' {
            let trunc = self
                .line_buffer
                .iter()
                .enumerate()
                .rev()
                .skip_while(|(_, b)| b == &&b' ')
                .next()
                .map(|(i, _)| i + 1)
                .unwrap_or_default();
            self.line_buffer.truncate(trunc);
            self.line_buffer.push(b'\n');
            self.flush()?;
            self.column = 0;
        }
        if buf.len() > 0 {
            self.is_start_of_file = false;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        self.w.write_all(&self.line_buffer)?;
        self.line_buffer.clear();
        self.w.flush()
    }
}

pub fn format_file(path: std::path::PathBuf, dry: bool, debug: DebugLang) -> std::io::Result<()> {
    let mut context = Context::new();
    context.debug = debug;
    let file = context.load_file(path.clone())?;
    if dry {
        let mut formatter = Formatter::new(stdout(), 100);
        let ast = compiler::parser::parse_file(file, &mut context);
        if debug.parser {
            debug_ast(&ast, &context);
        }
        format_ast(ast, &mut formatter, &mut context)?;
    } else {
        let mut formatter = Formatter::new(File::create(path)?, 100);
        let ast = compiler::parser::parse_file(file, &mut context);
        if debug.parser {
            debug_ast(&ast, &context);
        }
        format_ast(ast, &mut formatter, &mut context)?;
    }
    Ok(())
}

fn format_ast<W: Write + fmt::Debug>(
    ast: compiler::parser::Ast,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::io::Result<()> {
    for statement in ast.statements {
        format_node(statement, formatter, context)?;
    }
    formatter.emit_token(
        ast.eof,
        &context.loaded_files,
        TokenConfig::LEADING_BLANK_LINE,
    )?;
    // formatter.ensure_new_line()?;
    Ok(())
}

fn format_node<W: Write + fmt::Debug>(
    node: compiler::parser::SyntaxNode,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::io::Result<()> {
    match node.kind {
        SyntaxNodeKind::StylingStatement(styling_statement) => {
            format_styling_statement(styling_statement, formatter, context)
        }
        SyntaxNodeKind::ElementStatement(element_statement) => {
            format_element_statement(element_statement, formatter, context)
        }
        SyntaxNodeKind::TemplateStatement(template_statement) => {
            format_template_statement(template_statement, formatter, context)
        }
        SyntaxNodeKind::ImportStatement(import_statement) => {
            format_import_statement(import_statement, formatter, context)
        }
        SyntaxNodeKind::ExpressionStatement(expression_statement) => {
            format_expression_statement(expression_statement, formatter, context)
        }
        SyntaxNodeKind::VariableDeclaration(variable_declaration) => {
            format_variable_declaration(variable_declaration, formatter, context)
        }
        SyntaxNodeKind::SlideStatement(slide_statement) => {
            format_slide_statement(slide_statement, formatter, context)
        }
        SyntaxNodeKind::Literal(token) | SyntaxNodeKind::VariableReference(token) => {
            if matches!(token.kind, TokenKind::String) {
                format_string(token, formatter, context)
            } else {
                formatter.emit_token(token, &context.loaded_files, TokenConfig::TRIMMED)
            }
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
        SyntaxNodeKind::Error(true) => formatter.raw(node.location, &context.loaded_files),
        SyntaxNodeKind::Error(false) => Ok(()),
        SyntaxNodeKind::DictEntry(dict_entry) => format_dict_entry(dict_entry, formatter, context),
        SyntaxNodeKind::Dict(dict) => format_dict(dict, formatter, context),
        SyntaxNodeKind::Array(array) => format_array(array, formatter, context),
        SyntaxNodeKind::InferredMember(inferred_member) => todo!(),
        SyntaxNodeKind::PostInitialization(post_initialization) => {
            format_post_initialization(post_initialization, formatter, context)
        }
        SyntaxNodeKind::Parameter(parameter) => format_parameter(parameter, formatter, context),
        SyntaxNodeKind::ParameterBlock(parameter_block) => {
            format_parameter_block(parameter_block, formatter, context)
        }
        SyntaxNodeKind::Binary(binary) => todo!(),
    }
}

fn format_array<W: Write + fmt::Debug>(
    array: compiler::parser::Array,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::result::Result<(), std::io::Error> {
    let split = formatter.available_space()
        < Location::combine(array.lbracket.location, array.rbracket.location).length;
    formatter.emit_token(
        array.lbracket,
        &context.loaded_files,
        TokenConfig::TRAILING_SPACE,
    )?;
    formatter.indent += 4;
    for (expression, comma) in array.entries {
        if split {
            formatter.ensure_indented_line()?;
        }
        format_node(expression, formatter, context)?;
        if let Some(comma) = comma {
            formatter.emit_token(comma, &context.loaded_files, TokenConfig::TRAILING_SPACE)?;
        }
    }
    formatter.indent -= 4;
    formatter.ensure_space()?;
    formatter.emit_token(
        array.rbracket,
        &context.loaded_files,
        TokenConfig {
            leading_blank_line: split,
            ..Default::default()
        },
    )?;
    Ok(())
}

fn format_import_statement<W: Write + fmt::Debug>(
    import_statement: compiler::parser::ImportStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::result::Result<(), std::io::Error> {
    formatter.emit_token(
        import_statement.import_keyword,
        &context.loaded_files,
        TokenConfig::TRAILING_SPACE,
    )?;
    format_node(*import_statement.path, formatter, context)?;
    formatter.emit_token(
        import_statement.semicolon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    Ok(())
}

fn format_parameter<W: Write + fmt::Debug>(
    parameter: compiler::parser::Parameter,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> std::result::Result<(), std::io::Error> {
    formatter.emit_token(
        parameter.identifier,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        parameter.colon,
        &context.loaded_files,
        TokenConfig::TRAILING_SPACE,
    )?;
    formatter.emit_token(
        parameter.type_,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    Ok(())
}

fn format_parameter_block<W: Write + fmt::Debug>(
    parameter_block: compiler::parser::ParameterBlock,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(
        parameter_block.lparen,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    for (parameter, comma) in parameter_block.parameters {
        format_node(parameter, formatter, context)?;
        if let Some(comma) = comma {
            formatter.emit_token(comma, &context.loaded_files, TokenConfig::TRAILING_SPACE)?;
        }
    }
    formatter.emit_token(
        parameter_block.rparen,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    Ok(())
}

fn format_element_statement<W: Write + fmt::Debug>(
    element_statement: compiler::parser::ElementStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(
        element_statement.element_keyword,
        &context.loaded_files,
        TokenConfig {
            leading_blank_line: true,
            trailing_space: true,
            ..Default::default()
        },
    )?;
    formatter.emit_token(
        element_statement.name,
        &context.loaded_files,
        TokenConfig::default(),
    )?;

    format_node(*element_statement.parameters, formatter, context)?;

    formatter.emit_token(
        element_statement.colon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.indent += 4;
    for statement in element_statement.body {
        format_node(statement, formatter, context)?;
    }
    formatter.indent -= 4;
    Ok(())
}

fn format_template_statement<W: Write + fmt::Debug>(
    template_statement: compiler::parser::TemplateStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(
        template_statement.template_keyword,
        &context.loaded_files,
        TokenConfig {
            leading_blank_line: true,
            trailing_space: true,
            ..Default::default()
        },
    )?;
    formatter.emit_token(
        template_statement.name,
        &context.loaded_files,
        TokenConfig::default(),
    )?;

    format_node(*template_statement.parameters, formatter, context)?;

    formatter.emit_token(
        template_statement.colon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.indent += 4;
    for statement in template_statement.body {
        format_node(statement, formatter, context)?;
    }
    formatter.indent -= 4;
    Ok(())
}

fn format_expression_statement<W: Write + fmt::Debug>(
    expression_statement: compiler::parser::ExpressionStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.ensure_indented_line()?;
    format_node(*expression_statement.expression, formatter, context)?;
    formatter.emit_token(
        expression_statement.semicolon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.ensure_new_line()?;
    Ok(())
}

fn format_dict_entry<W: Write + fmt::Debug>(
    dict_entry: compiler::parser::DictEntry,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.ensure_indent()?;
    formatter.emit_token(
        dict_entry.identifier,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        dict_entry.colon,
        &context.loaded_files,
        TokenConfig::TRAILING_SPACE,
    )?;
    format_node(*dict_entry.value, formatter, context)?;
    Ok(())
}

fn format_dict<W: Write + fmt::Debug>(
    dict: compiler::parser::Dict,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(dict.lbrace, &context.loaded_files, TokenConfig::default())?;
    formatter.ensure_new_line()?;
    formatter.indent += 4;
    for (entry, comma) in dict.entries {
        format_node(entry, formatter, context)?;
        match comma {
            Some(it) => formatter.emit_token(it, &context.loaded_files, TokenConfig::default())?,
            None => write!(formatter, ",")?,
        }
        formatter.ensure_new_line()?;
    }
    formatter.indent -= 4;
    formatter.emit_token(dict.rbrace, &context.loaded_files, TokenConfig::default())?;
    Ok(())
}

fn format_post_initialization<W: Write + fmt::Debug>(
    post_initialization: compiler::parser::PostInitialization,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*post_initialization.expression, formatter, context)?;
    formatter.ensure_space()?;
    format_node(*post_initialization.dict, formatter, context)?;
    Ok(())
}

fn format_member_access<W: Write + fmt::Debug>(
    member_access: compiler::parser::MemberAccess,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*member_access.base, formatter, context)?;
    // formatter.reserve()
    formatter.emit_token(
        member_access.period,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        member_access.member,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    Ok(())
}

fn format_variable_declaration<W: Write + fmt::Debug>(
    variable_declaration: compiler::parser::VariableDeclaration,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(
        variable_declaration.let_keyword,
        &context.loaded_files,
        TokenConfig {
            trailing_space: true,
            ..Default::default()
        },
    )?;
    formatter.emit_token(
        variable_declaration.name,
        &context.loaded_files,
        TokenConfig::TRAILING_SPACE,
    )?;
    formatter.emit_token(
        variable_declaration.equals,
        &context.loaded_files,
        TokenConfig::TRAILING_SPACE,
    )?;
    formatter.indent += 4;
    let needed_space = variable_declaration.semicolon.location.end()
        - variable_declaration.expression.location.start;
    if formatter.available_space() < needed_space {
        formatter.ensure_indented_line()?;
    }
    // TODO: formatter.reserve(variable_declaration.expression.location.length)
    format_node(*variable_declaration.expression, formatter, context)?;
    formatter.emit_token(
        variable_declaration.semicolon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.indent -= 4;
    formatter.ensure_new_line()?;
    Ok(())
}

fn format_slide_statement<W: Write + fmt::Debug>(
    slide_statement: compiler::parser::SlideStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(
        slide_statement.slide_keyword,
        &context.loaded_files,
        TokenConfig {
            leading_blank_line: true,
            trailing_space: true,
            ..Default::default()
        },
    )?;
    formatter.emit_token(
        slide_statement.name,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        slide_statement.colon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.ensure_new_line()?;
    formatter.indent += 4;
    for statement in slide_statement.body {
        format_node(statement, formatter, context)?;
    }
    formatter.indent -= 4;
    // formatter.ensure_empty_line()?;
    Ok(())
}

fn format_string<W: Write + fmt::Debug>(
    token: Token,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    let string =
        Value::parse_string_literal(token.text(&context.loaded_files), false).into_string();
    formatter.ensure_indent()?;
    if string.contains('\n') {
        writeln!(formatter, "\"\"\"")?;
        formatter.indent += 4;
        formatter.ensure_indent()?;
    } else {
        write!(formatter, "\"")?;
    }
    write!(formatter, "{}", string)?;
    if string.contains('\n') {
        formatter.indent -= 4;
        formatter.ensure_indent()?;
        write!(formatter, "\n\"\"\"")?;
    } else {
        write!(formatter, "\"")?;
    }
    Ok(())
}

fn format_typed_string<W: Write + fmt::Debug>(
    typed_string: compiler::parser::TypedString,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    // TODO: Make it illegal to write `p "path"`
    formatter.emit_token(
        typed_string.type_,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    format_string(typed_string.string, formatter, context)?;

    // formatter.emit_token(
    //     typed_string.string,
    //     &context.loaded_files,
    //     TokenConfig::STRING,
    // )?;
    Ok(())
}

fn format_function_call<W: Write + fmt::Debug>(
    function_call: compiler::parser::FunctionCall,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    format_node(*function_call.base, formatter, context)?;
    formatter.emit_token(
        function_call.lparen,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    let split = formatter.available_space()
        < Location::combine(
            function_call
                .arguments
                .first()
                .map(|(a, _)| a.location)
                .unwrap_or(function_call.rparen.location),
            function_call.rparen.location,
        )
        .length;
    let arguments_count = function_call.arguments.len();
    formatter.indent += 4;
    for (i, (argument, comma)) in function_call.arguments.into_iter().enumerate() {
        if split {
            formatter.ensure_indented_line()?;
        }
        format_node(argument, formatter, context)?;
        if i != arguments_count - 1 {
            match comma {
                Some(it) => {
                    formatter.emit_token(it, &context.loaded_files, TokenConfig::TRAILING_SPACE)?
                }
                None => write!(formatter, ", ")?,
            }
        }
    }
    formatter.indent -= 4;
    formatter.emit_token(
        function_call.rparen,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    Ok(())
}

fn format_assignment_statement<W: Write + fmt::Debug>(
    assignment_statement: compiler::parser::AssignmentStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.ensure_indented_line()?;
    format_node(*assignment_statement.lhs, formatter, context)?;
    formatter.ensure_space()?;
    formatter.emit_token(
        assignment_statement.equals,
        &context.loaded_files,
        TokenConfig::TRAILING_SPACE,
    )?;
    format_node(*assignment_statement.assignment, formatter, context)?;
    formatter.emit_token(
        assignment_statement.semicolon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.ensure_new_line()?;
    Ok(())
}

fn format_styling_statement<W: std::io::Write + fmt::Debug>(
    styling_statement: compiler::parser::StylingStatement,
    formatter: &mut Formatter<W>,
    context: &mut Context,
) -> Result<()> {
    formatter.emit_token(
        styling_statement.styling_keyword,
        &context.loaded_files,
        TokenConfig {
            leading_blank_line: true,
            trailing_space: true,
            ..Default::default()
        },
    )?;
    formatter.emit_token(
        styling_statement.name,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        styling_statement.lparen,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        styling_statement.type_,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        styling_statement.rparen,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.emit_token(
        styling_statement.colon,
        &context.loaded_files,
        TokenConfig::default(),
    )?;
    formatter.indent += 4;
    formatter.ensure_new_line()?;
    for statement in styling_statement.body {
        format_node(statement, formatter, context)?;
    }
    formatter.indent -= 4;
    Ok(())
}
