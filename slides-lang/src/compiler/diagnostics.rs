use super::{
    binder::{Variable, typing::Type},
    lexer::Token,
};

use crate::{Files, Location};

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

pub struct Diagnostics {
    diagnostics: Vec<Diagnostic>,
}
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

    pub(crate) fn report_cannot_convert(&mut self, from: &Type, target: &Type, location: Location) {
        self.report_error(
            format!("Cannot convert type {from:?} to type {target:?}"),
            location,
        );
    }
}
