use ahash::AHashMap;
use itertools::Itertools;

use crate::{
    lexer::position::{Span, Spanned},
    parser::ast::DataTypeGetterRecursive,
};

use super::{
    ast::{
        Block, CommonGeneric, CustomDataType, DataType, DataTypeGetter, DataTypeSettable, Program,
    },
    error::ParserError,
};

pub fn same_variant<T>(enum1: &T, enum2: &T) -> bool
where
    T: std::cmp::PartialEq + std::fmt::Debug,
{
    std::mem::discriminant(enum1) == std::mem::discriminant(enum2)
}

pub fn specify_generics(
    type_settables: &mut [DataTypeSettable],
    generics: &AHashMap<String, DataType>,
    parser: &mut Program,
    handle_traits: bool,
) {
    // println!("INPUT GENERICS: {:?}", generics);
    for type_settable in type_settables {
        match type_settable {
            DataTypeSettable::DataType(ref mut type_) => match type_ {
                DataType::Generic(name) => {
                    if let Some(specific_type) = generics.get(name) {
                        // println!("SETTING GENERIC: {} -> {:?}", name, specific_type);
                        **type_ = specific_type.clone();
                    } else {
                        // **type_ = DataType::Generic(name.clone());
                        // unreachable!("Generic not found: {}; {:#?}", name, generics);
                    }
                }
                DataType::Custom(inner) => {
                    if inner.is_generic() {
                        let specific_types = generics
                            .iter()
                            .map(|(_, specific)| specific.clone())
                            .collect_vec();

                        let subtype = inner.subtype(&specific_types, parser, handle_traits);
                        *inner = subtype;
                    }
                }
                DataType::Pointer(inner) => {
                    // println!("Processing pointer type: {:?}", inner);

                    specify_generics(
                        &mut [DataTypeSettable::DataType(inner)],
                        generics,
                        parser,
                        handle_traits,
                    );
                }
                _ => {}
            },
            DataTypeSettable::FunctionCall(function, args) => {
                if function.is_generic() {
                    let mut type_settables = args
                        .iter_mut()
                        .map(DataTypeSettable::DataType)
                        .collect_vec();

                    type_settables
                        .push(DataTypeSettable::DataType(&mut function.return_type.value));

                    // println!(">> generics h: {:?}", generics);
                    // println!(">> type_settables: {:?}", type_settables);

                    specify_generics(&mut type_settables, generics, parser, true);

                    // println!(">> after specifiy: {:?}", type_settables);
                    let mut specific_types = args.clone();
                    specific_types.push(function.return_type.value.clone());
                    // println!(">> args after specifiy: {:?}", specific_types);

                    let subtype =
                        function.subtype(generics, function.method_of.as_ref(), parser, true);
                    **function = subtype;

                    parser.functions.insert(
                        function.name.clone(),
                        Spanned {
                            value: function.clone(),
                            span: Span::default(),
                        },
                    );
                }
            }
        }
    }
}

pub fn check_all_types_same<T>(types: &[Spanned<T>]) -> Result<DataType, Spanned<ParserError>>
where
    T: DataTypeGetter,
{
    let _type = types[0].value._type();

    for element in types {
        if element.value._type() != _type {
            return Err(Spanned {
                value: ParserError::WrongType(_type.clone(), element.value._type()),
                span: element.span,
            });
        }
    }

    Ok(_type)
}

#[allow(dead_code)]
fn find_specific_types(generic: CustomDataType, specific: CustomDataType) -> Option<Vec<DataType>> {
    if specific.subtype_of != Some(generic.name.clone()) {
        return None;
    }

    let mut out = vec![];
    let mut generics_to_specifics = AHashMap::new();

    let generic_types = generic.clone().types();
    let specific_types = specific.clone().types();

    for (generic, specific) in generic_types.iter().zip(specific_types.iter()) {
        match (generic, specific) {
            (DataType::Generic(g), specific) => {
                if !generics_to_specifics.contains_key(g) {
                    generics_to_specifics.insert(g.clone(), specific.clone());
                    out.push(specific.clone());
                }
            }
            (DataType::Custom(generic), DataType::Custom(specific)) => {
                if let Some(types) = find_specific_types(generic.clone(), specific.clone()) {
                    out.extend(types);
                }
            }
            _ => {}
        }
    }

    Some(out)
}
pub fn handle_generics(
    args: &[DataType],
    params: &[DataType],
    annotations: Option<&AHashMap<String, DataType>>,
) -> (Vec<DataType>, AHashMap<String, DataType>) {
    let mut generics_to_types = AHashMap::new();
    let mut generics_order = Vec::new();

    if let Some(annotations) = annotations {
        for (generic, specific) in annotations.iter() {
            generics_to_types.insert(generic.clone(), specific.clone());
            generics_order.push(generic.clone());
        }
    }

    for (arg, param) in args.iter().zip(params.iter()) {
        match param {
            DataType::Generic(g) => {
                if !generics_to_types.contains_key(g) {
                    generics_to_types.insert(g.clone(), arg.clone());
                    generics_order.push(g.clone());
                }
            }
            DataType::Custom(param) => {
                if let DataType::Custom(arg) = arg {
                    let inferred_types = handle_generics(
                        &arg.fields
                            .value
                            .iter()
                            .map(|f| f._type.value.clone())
                            .collect::<Vec<_>>(),
                        &param
                            .fields
                            .value
                            .iter()
                            .map(|f| f._type.value.clone())
                            .collect::<Vec<_>>(),
                        annotations,
                    );
                    for (g, t) in inferred_types.1.iter() {
                        if !generics_to_types.contains_key(g) {
                            generics_to_types.insert(g.clone(), t.clone());
                            generics_order.push(g.clone());
                        }
                    }
                }
            }
            DataType::Pointer(inner_param) => {
                if let DataType::Pointer(inner_arg) = arg {
                    let (_result, generics) =
                        handle_generics(&[*inner_arg.clone()], &[*inner_param.clone()], None);
                    for (g, t) in generics.iter() {
                        if !generics_to_types.contains_key(g) {
                            generics_to_types.insert(g.clone(), t.clone());
                            generics_order.push(g.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (
        generics_order
            .iter()
            .filter_map(|g| generics_to_types.get(g).cloned())
            .collect(),
        generics_to_types,
    )
}

/// um genestete generics in funktionsaufrufen richtig zu erstzen
/// bzw subtyp bilden
pub fn handle_nested_generic_functions(
    _block: &mut Block,
    _generics: &AHashMap<String, DataType>,
    _program: &mut Program,
) {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::FunctionParam;

    use super::*;

    #[test]
    fn test_specify_generics() {
        let mut binding1 = DataType::Generic("T".to_string());
        let mut binding2 = DataType::Pointer(Box::new(DataType::Generic("T".to_string())));
        let mut binding3 = DataType::Integer64;
        let mut binding4 = DataType::Generic("U".to_string());

        let mut settables = vec![
            DataTypeSettable::DataType(&mut binding1),
            DataTypeSettable::DataType(&mut binding2),
            DataTypeSettable::DataType(&mut binding3),
            DataTypeSettable::DataType(&mut binding4),
        ];

        let mut generics = AHashMap::new();
        generics.insert("T".to_string(), DataType::Integer64);
        generics.insert("U".to_string(), DataType::Boolean);

        let mut program = Program::default();

        specify_generics(&mut settables, &generics, &mut program, false);

        assert_eq!(binding1, DataType::Integer64);
        assert_eq!(binding2, DataType::Pointer(Box::new(DataType::Integer64)));
        assert_eq!(binding3, DataType::Integer64);
        assert_eq!(binding4, DataType::Boolean);
    }

    #[test]
    fn test_specify_generics_with_function() {
        let mut binding1 = DataType::Generic("T".to_string());
        let mut binding2 = DataType::Pointer(Box::new(DataType::Generic("T".to_string())));
        let mut binding3 = DataType::Integer64;
        // let mut binding4 =

        let _settables = [
            DataTypeSettable::DataType(&mut binding1),
            DataTypeSettable::DataType(&mut binding2),
            DataTypeSettable::DataType(&mut binding3),
        ];

        let mut generics = AHashMap::new();
        generics.insert("T".to_string(), DataType::Integer64);
    }

    #[test]
    fn handle_generics_test() {
        let args = vec![DataType::Integer64, DataType::Boolean, DataType::Float];

        let params = vec![
            DataType::Generic("T".to_string()),
            DataType::Generic("U".to_string()),
            DataType::Float,
        ];

        let (result, generics) = handle_generics(&args, &params, None);

        assert_eq!(result, vec![DataType::Integer64, DataType::Boolean]);

        let mut expected_generics = AHashMap::new();
        expected_generics.insert("T".to_string(), DataType::Integer64);
        expected_generics.insert("U".to_string(), DataType::Boolean);

        assert_eq!(generics, expected_generics);
    }

    #[test]
    fn handle_generics_test_2() {
        let args = vec![
            DataType::Integer64,
            DataType::Float,
            DataType::Integer64,
            DataType::Boolean,
        ];

        let params = vec![
            DataType::Generic("T".to_string()),
            DataType::Float,
            DataType::Generic("T".to_string()),
            DataType::Generic("U".to_string()),
        ];

        let (result, generics) = handle_generics(&args, &params, None);

        assert_eq!(result, vec![DataType::Integer64, DataType::Boolean]);

        let mut expected_generics = AHashMap::new();
        expected_generics.insert("T".to_string(), DataType::Integer64);
        expected_generics.insert("U".to_string(), DataType::Boolean);

        assert_eq!(generics, expected_generics);
    }

    #[test]
    fn handle_generics_test_3() {
        let args = vec![DataType::Pointer(Box::new(DataType::Integer64))];

        let params = vec![DataType::Pointer(Box::new(DataType::Generic(
            "E".to_string(),
        )))];

        let (result, generics) = handle_generics(&args, &params, None);

        assert_eq!(result, vec![DataType::Integer64]);

        let mut expected_generics = AHashMap::new();
        expected_generics.insert("E".to_string(), DataType::Integer64);

        assert_eq!(generics, expected_generics);
    }

    #[test]
    fn handle_generics_with_annotations() {
        // keine argumente
        let args: Vec<DataType> = vec![];

        let params = vec![
            // e.g the return type
            DataType::Pointer(Box::new(DataType::Generic("T".to_string()))),
        ];

        let mut annotations = AHashMap::new();
        annotations.insert("T".to_string(), DataType::Integer64);

        let (result, generics) = handle_generics(&args, &params, Some(&annotations));

        assert_eq!(result, vec![DataType::Integer64]);

        let mut expected_generics = AHashMap::new();
        expected_generics.insert("T".to_string(), DataType::Integer64);

        assert_eq!(generics, expected_generics);
    }

    #[test]
    fn handle_generics_with_classes() {
        let args: Vec<DataType> = vec![
            DataType::Custom(CustomDataType {
                display_name: "List<int64>".to_string(),
                name: "List--int64".to_string(),
                fields: Spanned {
                    value: vec![
                        FunctionParam {
                            name: Spanned {
                                value: "data".to_string(),
                                span: Span::default(),
                            },
                            _type: Spanned {
                                value: DataType::Pointer(Box::new(DataType::Integer64)),
                                span: Span::default(),
                            },
                        },
                        FunctionParam {
                            name: Spanned {
                                value: "len".to_string(),
                                span: Span::default(),
                            },
                            _type: Spanned {
                                value: DataType::Integer64,
                                span: Span::default(),
                            },
                        },
                        FunctionParam {
                            name: Spanned {
                                value: "cap".to_string(),
                                span: Span::default(),
                            },
                            _type: Spanned {
                                value: DataType::Integer64,
                                span: Span::default(),
                            },
                        },
                    ],
                    span: Span::default(),
                },
                subtypes: AHashMap::new(),
                generics: vec![],
                subtype_of: Some("List".to_string()),
                is_generic: false,
            }),
            DataType::Integer64,
        ];

        let params: Vec<DataType> = vec![DataType::Custom(CustomDataType {
            display_name: "List<T>".to_string(),
            name: "List".to_string(),
            fields: Spanned {
                value: vec![
                    FunctionParam {
                        name: Spanned {
                            value: "data".to_string(),
                            span: Span::default(),
                        },
                        _type: Spanned {
                            value: DataType::Pointer(Box::new(DataType::Generic("T".to_string()))),
                            span: Span::default(),
                        },
                    },
                    FunctionParam {
                        name: Spanned {
                            value: "len".to_string(),
                            span: Span::default(),
                        },
                        _type: Spanned {
                            value: DataType::Integer64,
                            span: Span::default(),
                        },
                    },
                    FunctionParam {
                        name: Spanned {
                            value: "cap".to_string(),
                            span: Span::default(),
                        },
                        _type: Spanned {
                            value: DataType::Integer64,
                            span: Span::default(),
                        },
                    },
                ],
                span: Span::default(),
            },
            subtypes: AHashMap::new(),
            generics: vec![Spanned {
                value: DataType::Generic("T".to_string()),
                span: Span::default(),
            }],
            subtype_of: None,
            is_generic: true,
        })];

        let (result, generics) = handle_generics(&args, &params, None);

        assert_eq!(result, vec![DataType::Integer64]);

        let mut expected = AHashMap::new();
        expected.insert("T".to_string(), DataType::Integer64);

        assert_eq!(generics, expected);
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         lexer::position::{Span, Spanned},
//         parser::ast::{CustomDataType, FunctionParam},
//     };

//     use super::*;
//     use pretty_assertions::assert_eq;

//     #[test]
//     fn test_find_specific_types() {
//         let generic = CustomDataType {
//             display_name: "Foo".to_string(),
//             name: "Foo".to_string(),
//             fields: Spanned {
//                 value: vec![FunctionParam {
//                     name: Spanned {
//                         value: "bar".to_string(),
//                         span: Span::default(),
//                     },
//                     _type: Spanned {
//                         value: DataType::Generic("T".to_string()),
//                         span: Span::default(),
//                     },
//                 }],
//                 span: Span::default(),
//             },
//             subtypes: Default::default(),
//             generics: vec![Spanned {
//                 value: DataType::Generic("T".to_string()),
//                 span: Span::default(),
//             }],
//             subtype_of: None,
//             is_generic: true,
//         };

//         let specfic = CustomDataType {
//             display_name: "Foo".to_string(),
//             name: "Foo".to_string(),
//             fields: Spanned {
//                 value: vec![FunctionParam {
//                     name: Spanned {
//                         value: "bar".to_string(),
//                         span: Span::default(),
//                     },
//                     _type: Spanned {
//                         value: DataType::Integer64,
//                         span: Span::default(),
//                     },
//                 }],
//                 span: Span::default(),
//             },
//             subtypes: Default::default(),
//             generics: vec![],
//             subtype_of: Some("Foo".to_string()),
//             is_generic: false,
//         };

//         let result = find_specific_types(generic, specfic);
//         assert_eq!(result, Some(vec![DataType::Integer64]))
//     }

//     #[test]
//     fn test_find_specific_types_2() {
//         let generic = CustomDataType {
//             display_name: "Foo".to_string(),
//             name: "Foo".to_string(),
//             fields: Spanned {
//                 value: vec![FunctionParam {
//                     name: Spanned {
//                         value: "bar".to_string(),
//                         span: Span::default(),
//                     },
//                     _type: Spanned {
//                         value: DataType::Custom(CustomDataType {
//                             display_name: "Foo".to_string(),
//                             name: "Foo".to_string(),
//                             fields: Spanned {
//                                 value: vec![
//                                     FunctionParam {
//                                         name: Spanned {
//                                             value: "bar".to_string(),
//                                             span: Span::default(),
//                                         },
//                                         _type: Spanned {
//                                             value: DataType::Generic("T".to_string()),
//                                             span: Span::default(),
//                                         },
//                                     },
//                                     FunctionParam {
//                                         name: Spanned {
//                                             value: "baz".to_string(),
//                                             span: Span::default(),
//                                         },
//                                         _type: Spanned {
//                                             value: DataType::Generic("U".to_string()),
//                                             span: Span::default(),
//                                         },
//                                     },
//                                 ],
//                                 span: Span::default(),
//                             },
//                             subtypes: Default::default(),
//                             generics: vec![
//                                 Spanned {
//                                     value: DataType::Generic("T".to_string()),
//                                     span: Span::default(),
//                                 },
//                                 Spanned {
//                                     value: DataType::Generic("U".to_string()),
//                                     span: Span::default(),
//                                 },
//                             ],
//                             subtype_of: Some("Foo".to_string()),
//                         }),
//                         span: Span::default(),
//                     },
//                 }],
//                 span: Span::default(),
//             },
//             subtypes: Default::default(),
//             generics: vec![],
//             subtype_of: None,
//             is_generic: todo!(),
//         };

//         let specfic = CustomDataType {
//             display_name: "Foo".to_string(),
//             name: "Foo".to_string(),
//             fields: Spanned {
//                 value: vec![FunctionParam {
//                     name: Spanned {
//                         value: "bar".to_string(),
//                         span: Span::default(),
//                     },
//                     _type: Spanned {
//                         value: DataType::Custom(CustomDataType {
//                             display_name: "Foo".to_string(),
//                             name: "Foo".to_string(),
//                             fields: Spanned {
//                                 value: vec![
//                                     FunctionParam {
//                                         name: Spanned {
//                                             value: "bar".to_string(),
//                                             span: Span::default(),
//                                         },
//                                         _type: Spanned {
//                                             value: DataType::Integer64,
//                                             span: Span::default(),
//                                         },
//                                     },
//                                     FunctionParam {
//                                         name: Spanned {
//                                             value: "baz".to_string(),
//                                             span: Span::default(),
//                                         },
//                                         _type: Spanned {
//                                             value: DataType::Boolean,
//                                             span: Span::default(),
//                                         },
//                                     },
//                                 ],
//                                 span: Span::default(),
//                             },
//                             subtypes: Default::default(),
//                             generics: vec![],
//                             subtype_of: Some("Foo".to_string()),
//                         }),
//                         span: Span::default(),
//                     },
//                 }],
//                 span: Span::default(),
//             },
//             subtypes: Default::default(),
//             generics: vec![],
//             subtype_of: Some("Foo".to_string()),
//         };

//         let result = find_specific_types(generic, specfic);
//         assert_eq!(result, Some(vec![DataType::Integer64, DataType::Boolean]))
//     }

//     #[test]
//     fn test_find_specific_types_3() {
//         let generic = CustomDataType {
//             display_name: "Foo".to_string(),
//             name: "Foo".to_string(),
//             fields: Spanned {
//                 value: vec![
//                     FunctionParam {
//                         name: Spanned {
//                             value: "bar".to_string(),
//                             span: Span::default(),
//                         },
//                         _type: Spanned {
//                             value: DataType::Generic("T".to_string()),
//                             span: Span::default(),
//                         },
//                     },
//                     FunctionParam {
//                         name: Spanned {
//                             value: "baz".to_string(),
//                             span: Span::default(),
//                         },
//                         _type: Spanned {
//                             value: DataType::Generic("T".to_string()),
//                             span: Span::default(),
//                         },
//                     },
//                 ],
//                 span: Span::default(),
//             },
//             subtypes: Default::default(),
//             generics: vec![Spanned {
//                 value: DataType::Generic("T".to_string()),
//                 span: Span::default(),
//             }],
//             subtype_of: None,
//         };

//         let specfic = CustomDataType {
//             display_name: "Foo".to_string(),
//             name: "Foo".to_string(),
//             fields: Spanned {
//                 value: vec![
//                     FunctionParam {
//                         name: Spanned {
//                             value: "bar".to_string(),
//                             span: Span::default(),
//                         },
//                         _type: Spanned {
//                             value: DataType::Integer64,
//                             span: Span::default(),
//                         },
//                     },
//                     FunctionParam {
//                         name: Spanned {
//                             value: "baz".to_string(),
//                             span: Span::default(),
//                         },
//                         _type: Spanned {
//                             value: DataType::Integer64,
//                             span: Span::default(),
//                         },
//                     },
//                 ],
//                 span: Span::default(),
//             },
//             subtypes: Default::default(),
//             generics: vec![],
//             subtype_of: Some("Foo".to_string()),
//         };

//         let result = find_specific_types(generic, specfic);
//         assert_eq!(result, Some(vec![DataType::Integer64]))
//     }

//     #[test]
//     fn test_inferred_generic_annotations() {
//         let args = vec![DataType::Integer64, DataType::Boolean, DataType::Float];

//         let params = vec![
//             DataType::Generic("T".to_string()),
//             DataType::Generic("U".to_string()),
//             DataType::Float,
//         ];

//         let result = inferred_generic_annotations(&args, &params);

//         assert_eq!(result, vec![DataType::Integer64, DataType::Boolean]);
//     }

//     #[test]
//     fn test_inferred_generic_annotations_2() {
//         let args = vec![
//             DataType::Integer64,
//             DataType::Float,
//             DataType::Integer64,
//             DataType::Boolean,
//         ];

//         let params = vec![
//             DataType::Generic("T".to_string()),
//             DataType::Float,
//             DataType::Generic("T".to_string()),
//             DataType::Generic("U".to_string()),
//         ];

//         let result = inferred_generic_annotations(&args, &params);

//         assert_eq!(result, vec![DataType::Integer64, DataType::Boolean])
//     }

//     #[test]
//     fn test_inferred_generic_annotations_3() {
//         let args = vec![
//             DataType::Custom(CustomDataType {
//                 display_name: "Foo".to_string(),
//                 name: "Foo__int".to_string(),
//                 fields: Spanned {
//                     value: vec![
//                         FunctionParam {
//                             name: Spanned {
//                                 value: "bar".to_string(),
//                                 span: Span::default(),
//                             },
//                             _type: Spanned {
//                                 value: DataType::Integer64,
//                                 span: Span::default(),
//                             },
//                         },
//                         FunctionParam {
//                             name: Spanned {
//                                 value: "baz".to_string(),
//                                 span: Span::default(),
//                             },
//                             _type: Spanned {
//                                 value: DataType::Boolean,
//                                 span: Span::default(),
//                             },
//                         },
//                     ],
//                     span: Span::default(),
//                 },
//                 subtypes: Default::default(),
//                 generics: vec![],
//                 subtype_of: Some("Foo".to_string()),
//             }),
//             DataType::Generic("T".to_string()),
//             DataType::Generic("U".to_string()),
//             DataType::Float,
//         ];

//         let params = vec![
//             DataType::Custom(CustomDataType {
//                 display_name: "Foo".to_string(),
//                 name: "Foo".to_string(),
//                 fields: Spanned {
//                     value: vec![
//                         FunctionParam {
//                             name: Spanned {
//                                 value: "bar".to_string(),
//                                 span: Span::default(),
//                             },
//                             _type: Spanned {
//                                 value: DataType::Generic("T".to_string()),
//                                 span: Span::default(),
//                             },
//                         },
//                         FunctionParam {
//                             name: Spanned {
//                                 value: "baz".to_string(),
//                                 span: Span::default(),
//                             },
//                             _type: Spanned {
//                                 value: DataType::Generic("U".to_string()),
//                                 span: Span::default(),
//                             },
//                         },
//                     ],
//                     span: Span::default(),
//                 },
//                 subtypes: Default::default(),
//                 generics: vec![
//                     Spanned {
//                         value: DataType::Generic("T".to_string()),
//                         span: Span::default(),
//                     },
//                     Spanned {
//                         value: DataType::Generic("U".to_string()),
//                         span: Span::default(),
//                     },
//                 ],
//                 subtype_of: None,
//             }),
//             DataType::Generic("T".to_string()),
//             DataType::Generic("U".to_string()),
//             DataType::Float,
//         ];

//         let result = inferred_generic_annotations(&args, &params);
//         assert_eq!(result, vec![DataType::Integer64, DataType::Boolean,])
//     }
// }
