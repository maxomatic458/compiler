use crate::lexer::position::Spanned;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use core::fmt::Debug;
use std::fmt::Display;

pub fn emit_error(file_name: &str, code: &str, err: &Spanned<Box<dyn CompilerError>>) {
    let mut files = SimpleFiles::new();

    let file_id = files.add(file_name, code);

    let diagnostic = Diagnostic::error()
        .with_message(err.value.name())
        .with_code(err.value.id().to_string())
        .with_labels(vec![Label::primary(
            file_id,
            err.span.start.abs..err.span.end.abs,
        )
        .with_message(err.value.err_msg())]);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config {
        before_label_lines: 2,
        after_label_lines: 2,
        ..Default::default()
    };

    codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
}

pub trait CompilerError: Debug + Display + std::error::Error + Send + Sync {
    fn id(&self) -> u32;
    fn name(&self) -> &str;
    fn err_msg(&self) -> String;
}
