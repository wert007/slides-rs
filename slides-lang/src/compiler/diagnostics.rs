use super::{FileId, Files, lexer::Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub file: FileId,
    pub start: usize,
    pub length: usize,
}
impl Location {
    pub fn set_end(&mut self, end: usize) {
        self.length = end - self.start;
    }

    pub(crate) fn combine(start: Location, end: Location) -> Self {
        Self {
            file: start.file,
            start: start.start,
            length: end.end() - start.start,
        }
    }

    fn end(&self) -> usize {
        self.start + self.length
    }
}

pub struct Diagnostic {
    error_message: String,
    location: Location,
}
impl Diagnostic {
    fn write<W: std::io::Write>(&self, w: &mut W, files: &Files) -> std::io::Result<()> {
        let file = &files[self.location.file];
        let file_name = file.name.display();
        let line_number = file.line_number(self.location.start);
        writeln!(w, "[{file_name}:{line_number}] {}", self.error_message)?;
        Ok(())
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

    fn report_error(&mut self, error_message: String, location: Location) {
        self.diagnostics.push(Diagnostic {
            error_message,
            location,
        });
    }

    pub fn report_unexpected_char(&mut self, unexpected: char, location: Location) {
        self.report_error(format!("Unexpected char `{unexpected}` found"), location);
    }

    pub fn report_expected_expression(&mut self, token: Token, files: &super::Files) {
        self.report_error(
            format!("Expected expression, found `{}` instead", token.text(files)),
            token.location,
        );
    }

    pub fn report_invalid_top_level_statement(&mut self, token: Token, files: &super::Files) {
        self.report_error(
            format!(
                "Expected either a slide or a styling, found `{}` instead",
                token.text(files)
            ),
            token.location,
        );
    }

    pub(crate) fn write<W: std::io::Write>(
        self,
        w: &mut W,
        files: &super::Files,
    ) -> std::io::Result<()> {
        for diagnostic in self.diagnostics {
            diagnostic.write(w, files)?;
        }
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
