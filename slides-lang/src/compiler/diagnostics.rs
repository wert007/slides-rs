use std::{error::Error, fmt::Display};

use super::{
    binder::{Variable, typing::Type},
    lexer::Token,
};

use crate::{Files, Location, StringInterner, compiler::binder::typing::TypeInterner};

#[derive(Debug)]
pub struct Diagnostic {
    error_message: String,
    location: Location,
    hints: Vec<Diagnostic>,
}
impl Diagnostic {
    fn write<W: std::io::Write>(self, w: &mut W, files: &Files) -> std::io::Result<()> {
        let file = &files[self.location.file];
        let file_name = file.name.display();
        let line_number = file.line_number(self.location.start);
        writeln!(w, "[{file_name}:{line_number}] {}", self.error_message)?;
        for hint in self.hints {
            hint.write(w, files)?;
        }
        Ok(())
    }

    fn add_hint(&mut self, message: String, location: Location) -> &mut Self {
        self.hints.push(Diagnostic {
            error_message: message,
            location,
            hints: Vec::new(),
        });
        self
    }
}

#[derive(Debug)]
pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}

impl Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Diagnotics ({})", self.diagnostics.len())
    }
}

impl Error for Diagnostics {}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    fn report_error(&mut self, error_message: String, location: Location) -> &mut Diagnostic {
        self.diagnostics.push(Diagnostic {
            error_message,
            location,
            hints: Vec::new(),
        });
        self.diagnostics.last_mut().unwrap()
    }

    pub fn report_unexpected_char(&mut self, unexpected: char, location: Location) {
        self.report_error(format!("Unexpected char `{unexpected}` found"), location);
    }

    pub fn report_expected_expression(&mut self, token: Token, files: &Files) {
        self.report_error(
            format!("Expected expression, found `{}` instead", token.text(files)),
            token.location,
        );
    }

    pub fn report_invalid_top_level_statement(&mut self, token: Token, files: &Files) {
        self.report_error(
            format!(
                "Expected either a slide or a styling, found `{}` instead",
                token.text(files)
            ),
            token.location,
        );
    }

    pub(crate) fn write<W: std::io::Write>(self, w: &mut W, files: &Files) -> std::io::Result<()> {
        for diagnostic in self.diagnostics {
            diagnostic.write(w, files)?;
        }
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub(crate) fn report_redeclaration_of_variable(
        &mut self,
        location: Location,
        name: &str,
        previous: &Variable,
    ) {
        self.report_error(
            format!("Unallowed redeclaration of variable {name}."),
            location,
        )
        .add_hint(
            format!("Previous declaration was here"),
            previous.definition,
        );
    }

    pub(crate) fn report_unexpected_styling_type(&mut self, type_: &str, location: Location) {
        self.report_error(format!("Unexpected styling type {type_}"), location);
    }

    pub(crate) fn report_unknown_member(
        &mut self,
        location: Location,
        base_type: &Type,
        name: &str,
    ) {
        self.report_error(
            format!("Unknown member {name} on Type {base_type:?}"),
            location,
        );
    }

    pub(crate) fn report_unknown_string_type(&mut self, string_type: &str, location: Location) {
        self.report_error(
            format!("Unknown string_type {string_type} found."),
            location,
        );
    }

    pub(crate) fn report_unknown_variable(&mut self, location: Location, variable: &str) {
        self.report_error(format!("No variable named {variable} found"), location);
    }

    pub(crate) fn report_cannot_convert(
        &mut self,
        type_interner: &TypeInterner,
        string_interner: &StringInterner,
        from: &Type,
        target: &Type,
        location: Location,
    ) {
        let from = type_interner.to_simple_string(from, string_interner);
        let target = type_interner.to_simple_string(target, string_interner);
        self.report_error(
            format!("Cannot convert type {from} to type {target}"),
            location,
        );
    }

    pub fn report_unknown_type(&mut self, location: Location, type_: &str) {
        self.report_error(format!("No Type named {type_} found"), location);
    }

    pub(crate) fn report_wrong_argument_count(
        &mut self,
        location: Location,
        function_type: super::binder::typing::FunctionType,
        actual_argument_count: usize,
    ) {
        self.report_error(
            format!(
                "Expected {} arguments, but found {actual_argument_count} instead",
                function_type.argument_count()
            ),
            location,
        );
    }

    pub(crate) fn report_unexpected_token(
        &mut self,
        actual: Token,
        expected: super::lexer::TokenKind,
    ) {
        self.report_error(
            format!(
                "Expected a {:?} but actually found a {:?}",
                expected, actual.kind
            ),
            actual.location,
        );
    }

    pub(crate) fn report_invalid_binary_operation(
        &mut self,
        lhs_type: &Type,
        operator: super::binder::BoundBinaryOperator,
        rhs_type: &Type,
        location: Location,
    ) {
        self.report_error(
            format!(
                "Invalid binary operation: {lhs_type:?} {} {rhs_type:?}",
                operator.to_string()
            ),
            location,
        );
    }

    pub(crate) fn report_field_does_not_exist(
        &mut self,
        location: Location,
        string_interner: &StringInterner,
        struct_data: &super::binder::typing::StructData,
        field_name: crate::VariableId,
    ) {
        //
        // let available = struct_data
        //     .fields
        //     .keys()
        //     .map(|k| context.string_interner.resolve_variable(*k))
        //     .collect::<Vec<_>>()
        //     .join(", ");
        // eprintln!(
        //     "TOO MANY FIELDS! found {}, but these are available: {available}",
        //     context.string_interner.resolve_variable(*field_name)
        // );
        self.report_error(
            format!(
                "struct {} has no field named {}.",
                string_interner.resolve_variable(struct_data.name),
                string_interner.resolve_variable(field_name)
            ),
            location,
        );
    }
}
