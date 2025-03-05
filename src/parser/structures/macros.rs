use crate::{
    lexer::{
        lexer_main::lex,
        position::Spanned,
        tokens::{Punctuation, Token},
    },
    parser::{
        ast::{Block, Expr, Program, TypedExpr},
        error::ParserError,
        parser_main::Parser,
        utils::check_all_types_same,
    },
};
use itertools::Itertools;
use unescape::unescape;

impl Parser {
    /// ein macro sind im compiler definierte schreibweisen um ein objekt in der sprache zu konstruiren
    /// es wird vorrausgesetzt das diese objekt definiert ist
    /// bsp
    /// String: "test" -> String::new(...)
    /// List: list![1, 2, 3] -> List::new<T>(...)
    ///
    /// implementiere wie rust macro bzw zu block
    /// let list = list![1, 2, 3]
    /// wird zu
    /// let list = {
    ///     let list = List::new<int>();
    ///     list.push(1);
    ///     list.push(2);
    ///     list.push(3);
    ///     return list;
    /// }

    pub(in crate::parser) fn parse_macro(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        match self.peek()?.value {
            Token::String(_) => self.parse_string_macro(scope),
            Token::MacroKeyword(macro_name) if macro_name == *"list!" => {
                self.parse_list_macro(scope)
            }

            unexpected => Err(Spanned {
                value: ParserError::UnexpectedToken(unexpected),
                span: self.peek()?.span,
            }),
        }
    }

    fn parse_string_macro(
        &mut self,
        _scope: &Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        const STRING_STRUCT_REQ: &str = "String";
        const STRING_METHOD_REQ: [&str; 3] = ["new", "with_capacity", "push_char"];

        if let Spanned {
            value: Token::String(string),
            span,
        } = self.next_token()?
        {
            check_macro_requirements(self, (STRING_STRUCT_REQ, &STRING_METHOD_REQ))
                .map_err(|err| Spanned { value: err, span })?;

            let ascii_letters = unescape(&string)
                .unwrap()
                .chars()
                .map(|c| c as u8)
                .collect_vec();

            let code = format!(
                "
                {{
                    let s = String::with_capacity({});
                    {}
                    return s;
                }}",
                ascii_letters.len(),
                ascii_letters
                    .iter()
                    .map(|c| format!("s.push_char({} as int8);", c))
                    .join("\n")
            );

            return Ok(Spanned {
                value: block_parse(&code, &mut self.program),
                span,
            });
        }

        unreachable!()
    }

    fn parse_list_macro(
        &mut self,
        scope: &mut Block,
    ) -> Result<Spanned<TypedExpr>, Spanned<ParserError>> {
        const LIST_STRUCT_REQ: &str = "List";
        const LIST_METHOD_REQ: [&str; 2] = ["new", "push"];

        let mut span = self.next_token()?.span; // "list!"
        span.extend(
            &self
                .expect_next(&[Token::Punctuation(Punctuation::OpenBracket)])?
                .span,
        ); // "["
        check_macro_requirements(self, (LIST_STRUCT_REQ, &LIST_METHOD_REQ))
            .map_err(|err| Spanned { value: err, span })?;

        let mut elements = vec![];
        // um iterator nicht zu verändern TODO: vielleicht brauch man das nicht
        let mut dummy_parser = self.clone();

        let end = dummy_parser.walk_separated_values(
            Token::Punctuation(Punctuation::Comma),
            Token::Punctuation(Punctuation::CloseBracket),
            |parser| {
                elements.push(parser.parse_expression(scope)?);

                Ok(())
            },
        )?;

        span = span.extend(&end);

        if elements.is_empty() {
            return Err(Spanned {
                value: ParserError::MacroError(
                    "Cannot infer type of empty list, consider using `List::new<T>()`".to_string(),
                ),
                span,
            });
        }

        let _type = check_all_types_same(&elements)?;

        let mut element_strings = Vec::with_capacity(elements.len());

        for element in &elements {
            let element_string = element.value.raw.clone().unwrap_or(
                self.program.source_code[element.span.start.abs..element.span.end.abs]
                    .iter()
                    .collect::<String>(),
            );
            element_strings.push(element_string);
        }

        let list_name = format!("list_{}", self.get_count()); //TODO:??

        let code = format!(
            "
            {{
                let {list_name} = List::new<{}>();
                {}
                return {list_name};
            }}",
            _type,
            element_strings
                .iter()
                .map(|s| format!("{list_name}.push({});", s))
                .join("\n")
        );

        let block = block_parse(&code, &mut dummy_parser.program);

        self.program = dummy_parser.program.clone();
        self.tokens = dummy_parser.tokens;

        Ok(Spanned { value: block, span })
    }
}

// TODO: reqs auch mit funktionsargumenten sonst möglicher panic :221
fn check_macro_requirements(parser: &mut Parser, reqs: (&str, &[&str])) -> Result<(), ParserError> {
    let (class, methods) = reqs;

    // TODO: builtins + methoden argumente
    if let Some(Spanned {
        value: class,
        span: _,
    }) = parser.program.custom_types.get(class).cloned()
    {
        let data_type_info = parser.get_type_info(&class);
        for method in methods {
            if !data_type_info.methods.iter().any(|m| m == method) {
                return Err(ParserError::MacroError(format!(
                    "the method '{}' is not defined in the struct '{}'",
                    method, class
                )));
            }
        }
    } else {
        return Err(ParserError::MacroError(format!(
            "the struct '{}' is not defined",
            class
        )));
    }

    Ok(())
}

// fn parse_expression(code: &str)

fn block_parse(code: &str, ctx: &mut Program) -> TypedExpr {
    let tokens = lex(code).unwrap();
    let mut parser = Parser::new(tokens, None).with_source_code(code);
    parser.program = ctx.clone();
    let mut scope = Block::default();
    // hier vielleicht panic
    let block = parser.parse_block(&mut scope).unwrap();
    let _span = block.span;
    let _type = block.value.return_type.clone();

    *ctx = parser.program.clone();
    TypedExpr {
        expression: Expr::Block { body: block },
        _type,
        raw: Some(code.to_string()),
    }
}
