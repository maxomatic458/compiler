use ahash::AHashMap;
use indexmap::IndexMap;

use crate::lexer::position::{Span, Spanned};

use super::ast::{DataType, Function, FunctionParam};

pub fn get_builtin_functions() -> IndexMap<String, Spanned<Function>> {
    let size_of = Function {
        display_name: Spanned {
            value: "size_of".to_string(),
            span: Span::default(),
        },
        name: "size_of".to_string(),
        params: Spanned {
            value: vec![Spanned {
                value: FunctionParam {
                    name: Spanned {
                        value: "type".to_string(),
                        span: Span::default(),
                    },
                    _type: Spanned {
                        value: DataType::DataType,
                        span: Span::default(),
                    },
                },
                span: Span::default(),
            }],
            span: Span::default(),
        },
        // local_variables: HashMap::new(),
        body: Spanned {
            value: Default::default(),
            span: Span::default(),
        },
        return_type: Spanned {
            value: DataType::get_integer_type(),
            span: Span::default(),
        },
        is_extern: true,
        method_of: None,
        is_builtin: true,
        // generics: vec![],
        generic_subtypes: AHashMap::new(),
        trait_of: None,
    };

    let mut map = IndexMap::new();
    map.insert(
        size_of.display_name.value.clone(),
        Spanned {
            value: size_of,
            span: Span::default(),
        },
    );
    map
}
