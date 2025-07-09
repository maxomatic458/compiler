use ordermap::OrderSet;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    mem::size_of,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::lexer::{
    position::{Span, Spanned},
    tokens::Literal,
};
use derive_more::Display;
use itertools::Itertools;
use ordermap::OrderMap;

use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use super::{
    builtins::get_builtin_functions,
    error::ParserError,
    parser_main::Parser,
    utils::{same_variant, specify_generics},
};

/// Trait, Parameter der Methode, Name der Methode (falls überschrieben), Rückgabetyp
type TraitInfo = (Trait, Vec<DataType>, Option<String>, DataType);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

// https://github.com/jDomantas/plank/blob/master/plank-syntax/src/ast.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub data_types: OrderMap<String, DataTypeInfo>,
    pub custom_types: OrderMap<String, Spanned<DataType>>,
    pub functions: OrderMap<String, Spanned<Function>>,
    pub require_main: bool,
    #[serde(skip)]
    pub dependency_cache: Arc<RwLock<HashMap<PathBuf, Program>>>,
    #[serde(skip)]
    pub import_queue: Vec<PathBuf>,
    #[serde(skip)]
    pub source_code: Vec<char>,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            data_types: OrderMap::new(),
            custom_types: OrderMap::new(),
            functions: get_builtin_functions(),
            require_main: false,
            dependency_cache: Arc::new(RwLock::new(HashMap::new())),
            import_queue: vec![],
            source_code: vec![],
        }
    }
}

impl Program {
    pub fn add_dependency_to_queue(&mut self, path: PathBuf) {
        self.import_queue.push(path);
    }

    pub fn pop_dependency_queue(&mut self) -> Option<PathBuf> {
        self.import_queue.pop()
    }

    pub fn cache_dependency(&mut self, path: PathBuf, program: Program) {
        self.dependency_cache.write().unwrap().insert(path, program);
    }
    /// Sperrt Arc<Mutex<T>>
    pub fn get_cached_dependency(&self, path: &Path) -> Option<Program> {
        self.dependency_cache.write().unwrap().get(path).cloned()
    }

    pub fn with_source_code(mut self, source_code: Vec<char>) -> Self {
        self.source_code = source_code;
        self
    }

    pub fn with_require_main(mut self, require_main: bool) -> Self {
        self.require_main = require_main;
        self
    }

    pub fn get_type_info(&self, _type: &DataType) -> DataTypeInfo {
        self.data_types
            .get(&_type.internal_name())
            .cloned()
            .unwrap_or(DataTypeInfo {
                parent_type: _type.clone(),
                methods: vec![],
                traits: HashSet::new(),
                // export:
            })
    }

    pub fn get_type_info_mut(&mut self, _type: &DataType) -> &mut DataTypeInfo {
        self.data_types
            .entry(_type.internal_name())
            .or_insert_with(|| DataTypeInfo {
                parent_type: _type.clone(),
                methods: vec![],
                traits: HashSet::new(),
            })
    }

    pub fn get_trait_function(
        &self,
        type_info: &DataTypeInfo,
        trait_: &Trait,
        params: &[DataType],
    ) -> Option<Function> {
        let function_name = type_info.get_trait_override_function_name(trait_, params);
        function_name
            .map(|function_name| self.functions.get(&function_name).cloned().unwrap().value)
    }

    // pub fn get_trait_return_type(&self, type_info: &DataTypeInfo, trait_: &Trait, params: &[DataType]) -> DataType {
    //     type_info.get_default_trait_return_type(trait_, params)
    //         .unwrap_or_else(|| self.get_trait_function(type_info, trait_, params).unwrap().return_type.value)
    // }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Function {
    pub display_name: Spanned<String>,
    pub name: String,
    pub params: Spanned<Vec<Spanned<FunctionParam>>>,
    // pub local_variables: AHashMap<String, Spanned<Variable>>,
    pub body: Spanned<Block>,
    pub return_type: Spanned<DataType>,
    pub is_extern: bool,
    pub method_of: Option<DataType>,
    pub trait_of: Option<DataType>,
    pub generic_subtypes: HashMap<Vec<DataType>, Function>,
    // pub generics: Vec<Spanned<DataType>>,
    pub is_builtin: bool,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) -> {}",
            self.display_name.value,
            self.params
                .value
                .iter()
                .map(|p| p.value._type.value.clone().to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.return_type.value
        )
    }
}

// closure

impl Function {
    pub fn get_variable(
        &self,
        name: &Spanned<String>,
    ) -> Result<Spanned<Variable>, Spanned<ParserError>> {
        self.body.value.get_variable(name)
    }

    pub fn import_compare(&self, other: &Function) -> bool {
        // self.display_name == other.display_name
        self.name == other.name
            && self.params == other.params
            && self.return_type == other.return_type
            // && self.body == other.body
            && self.is_extern == other.is_extern
            && self.method_of == other.method_of
            && self.trait_of == other.trait_of
        // && self.generic_subtypes == other.generic_subtypes
    }

    pub fn from_block(block: Block, name: String) -> Function {
        Function {
            display_name: Spanned {
                ..Default::default()
            },
            name,
            params: Spanned {
                value: block.closure_params.clone(),
                span: Span::default(),
            },
            body: Spanned {
                value: block.clone(),
                span: Span::default(),
            },
            return_type: Spanned {
                value: block.return_type,
                span: Span::default(),
            },
            is_extern: false,
            method_of: None,
            generic_subtypes: HashMap::new(),
            is_builtin: false,
            trait_of: None,
        }
    }

    pub fn is_static_method(&self) -> bool {
        self.is_method()
            && if let Some(self_param) = self.params.value.first() {
                self_param.value.name.value != *CLASS_SELF_ARG_NAME
            } else {
                true
            }
    }

    pub fn is_method(&self) -> bool {
        self.method_of.is_some()
    }

    pub fn return_type(&self) -> DataType {
        self.body.value.return_type.clone()
    }

    pub fn to_call(&self, params: &[TypedExpr]) -> TypedExpr {
        TypedExpr {
            expression: Expr::Call {
                function: Spanned {
                    value: self.clone(),
                    span: Span::default(),
                },
                args: Spanned {
                    value: params
                        .iter()
                        .map(|p| Spanned {
                            value: CallArg(p.clone()),
                            span: Span::default(),
                        })
                        .collect(),
                    span: Span::default(),
                },
            },
            _type: self.return_type.clone().value,
            raw: None,
        }
    }

    pub fn generic_param_count(&self) -> usize {
        self.body.value.generics.len()
    }

    pub fn subtype(
        &self,
        generic_map: &HashMap<String, DataType>,
        caller: Option<&DataType>,
        parser: &mut Program,
        handle_traits: bool,
    ) -> Function {
        let mut all_types = vec![];

        let mut params = self.params.clone();
        let mut param_types: Vec<DataTypeSettable> = params
            .value
            .iter_mut()
            .flat_map(|p| p.value._type.value._type_mut())
            .collect();
        all_types.append(&mut param_types);

        let mut body = self.body.clone();
        body.value.generics.clear();
        all_types.append(&mut body.value._type_mut());

        let mut generics = self.body.value.generics.clone();
        all_types.append(
            &mut generics
                .iter_mut()
                .flat_map(|g| g.value._type_mut())
                .collect::<Vec<_>>(),
        );

        // let mut return_type = body.value.return_type.clone();
        // all_types.push(DataTypeSettable::DataType(&mut return_type));

        specify_generics(&mut all_types, generic_map, parser, handle_traits);

        let name = format!(
            "{}--{}",
            self.name,
            generic_map
                .iter()
                .map(|(_, s)| s.internal_name())
                .collect::<Vec<_>>()
                .join(".")
        );

        let mut method_of = self.method_of.clone();

        if let Some(DataType::Custom(custom_type)) = caller {
            if !self.is_static_method() {
                method_of = Some(DataType::Custom(custom_type.clone()));
            }
        }

        let generics = all_types
            .iter()
            .filter_map(|t| {
                if let DataTypeSettable::DataType(t) = t {
                    Some((*t).clone())
                } else {
                    None
                }
            })
            .filter(|t| matches!(t, DataType::Generic(_)))
            .collect::<Vec<_>>();

        if !generics.is_empty() {
            body.value.generics = generics
                .iter()
                .map(|t| Spanned {
                    value: (*t).clone(),
                    span: Span::default(),
                })
                .collect();
        }

        let return_type = body.value.return_type.clone();

        Function {
            display_name: self.display_name.clone(),
            name: name.clone(),
            params,
            body,
            return_type: Spanned {
                value: return_type,
                span: Span::default(),
            },
            is_extern: self.is_extern,
            method_of,
            generic_subtypes: HashMap::new(),
            is_builtin: self.is_builtin,
            trait_of: None,
        }
        // println!("function.subtype SPECIFIC TYPES: {:?}", specific_types);
        // println!("function.caller: {:?}", caller.map(|c| c.to_string()));
        // println!("function.subtype: {:?}", self.body.value.generics.iter().map(|g| g.value.to_string()).collect::<Vec<_>>());

        // let mut generic_map: AHashMap<String, DataType> = AHashMap::new();

        // for (generic, specific) in self.body.value.generics.iter().map(|g| g.value.clone()).zip(specific_types.iter().cloned()) {
        //     if let DataType::Generic(generic_name) = generic {
        //         if !generic_map.contains_key(&generic_name) {
        //             generic_map.insert(generic_name, specific.to_owned());
        //         }
        //     }
        // }

        // for generic in self.body.value.generics.iter().filter_map(|g| if let DataType::Generic(g) = g.value.clone() { Some(g) } else { None }) {
        //     if !generic_map.contains_key(&generic) {
        //         generic_map.insert(generic.clone(), DataType::Generic(generic.clone()));
        //     }
        // }

        // println!("function.subtype. GENERIC MAP: {:#?}", generic_map);

        // let mut all_types = vec![];

        // let mut params = self.params.clone();
        // let mut param_types: Vec<DataTypeSettable> = params
        //     .value
        //     .iter_mut()
        //     .flat_map(|p| p.value._type.value._type_mut())
        //     .collect();

        // all_types.append(&mut param_types);

        // let return_type = &mut self.return_type.value.clone(); // ?!? TODO

        // all_types.push(DataTypeSettable::DataType(return_type));

        // let mut body = self.body.clone();
        // body.value.generics.clear();

        // all_types.append(&mut body.value._type_mut());

        // let mut local_variables = self.body.value.variables.clone();

        // for variable in local_variables.values_mut() {
        //     let types = &mut variable.value._type._type_mut();
        //     all_types.append(types);
        // }

        // println!("function.subtype. ALL TYPES: {:#?}", all_types);

        // specify_generics(&mut all_types, &generic_map, parser, handle_traits);

        // let name = format!(
        //     "{}--{}",
        //     self.name,
        //     generic_map
        //         .iter()
        //         .map(|(_, s)| s.internal_name())
        //         .collect::<Vec<_>>()
        //         .join(".")
        // );

        // let mut method_of = self.method_of.clone();

        // if let Some(DataType::Custom(custom_type)) = caller {
        //     if !self.is_static_method() {
        //         method_of = Some(DataType::Custom(custom_type.clone()));
        //     }
        // }

        // let generics: Vec<DataType> = all_types
        //     .iter()
        //     .filter_map(|t| {
        //         if let DataTypeSettable::DataType(t) = t {
        //             Some((*t).clone())
        //         } else {
        //             None
        //         }
        //     })
        //     .filter(|t| t.is_generic())
        //     .collect();

        // println!("function.subtype. NEW GENERICS: {:#?}", generics);

        // if !generics.is_empty() {
        //     body.value.generics = generics
        //         .iter()
        //         .map(|t| Spanned {
        //             value: (*t).clone(),
        //             span: Span::default(),
        //         })
        //         .collect();
        // }

        // // body.value.generics.clear();

        // // let mut body = self.body.clone();

        // let f = Function {
        //     display_name: self.display_name.clone(),
        //     name: name.clone(),
        //     params,
        //     body,
        //     return_type: Spanned {
        //         value: return_type.clone(),
        //         span: Span::default(),
        //     },
        //     is_extern: self.is_extern,
        //     method_of,
        //     generic_subtypes: AHashMap::new(),
        //     is_builtin: self.is_builtin,
        //     trait_of: None,
        // };

        // if name.starts_with("_realloc") {
        //     println!("{}: {}", name, f);
        // }

        // f
    }
}

impl CommonGeneric for Function {
    fn is_generic(&self) -> bool {
        !self.body.value.generics.is_empty()
    }

    fn generics(&self) -> OrderSet<String> {
        self.body
            .value
            .generics
            .iter()
            .map(|g| {
                if let DataType::Generic(name) = &g.value {
                    name
                } else {
                    unreachable!()
                }
            })
            .cloned()
            .collect()
    }
}

impl Default for Function {
    fn default() -> Self {
        Function {
            display_name: Spanned {
                value: "".to_string(),
                span: Span::default(),
            },
            params: Spanned {
                value: vec![],
                span: Span::default(),
            },
            body: Spanned {
                value: Block::default(),
                span: Span::default(),
            },
            return_type: Spanned {
                value: DataType::Integer32,
                span: Span::default(),
            },
            is_extern: false,
            method_of: None,
            generic_subtypes: HashMap::new(),
            is_builtin: false,
            name: "".to_string(),
            trait_of: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Hash)]
pub struct FunctionParam {
    pub name: Spanned<String>,
    pub _type: Spanned<DataType>,
}

impl From<Variable> for FunctionParam {
    fn from(value: Variable) -> Self {
        FunctionParam {
            name: value.name,
            _type: Spanned {
                value: value._type,
                span: Span::default(),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ElseIfBranch {
    pub condition: Spanned<TypedExpr>,
    pub body: Spanned<Block>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Statement {
    If {
        condition: Spanned<TypedExpr>,
        true_branch: Box<Spanned<Block>>,
        else_if_branches: Vec<Spanned<ElseIfBranch>>,
        false_branch: Option<Box<Spanned<Block>>>,
    },
    WhileLoop {
        condition: Spanned<TypedExpr>,
        body: Box<Spanned<Block>>,
    },
    VariableDecl {
        is_mutable: bool,
        name: Spanned<String>,
        _type: Option<Spanned<DataType>>,
        value: Spanned<TypedExpr>,
    },
    Return {
        value: Spanned<TypedExpr>,
    },
    Expr(Spanned<Expr>), // nur funktionsaufruf
    VariableMutation {
        variable: Spanned<TypedExpr>,
        new_value: Spanned<TypedExpr>,
    },
}

impl Statement {
    pub fn walk<F>(&self, f: &mut F)
    where
        F: FnMut(&Statement),
    {
        f(self);
        match self {
            Statement::If {
                true_branch,
                false_branch,
                ..
            } => {
                true_branch.value.walk(f);

                if let Some(false_branch) = false_branch {
                    false_branch.value.walk(f);
                }
            }
            Statement::WhileLoop { body, .. } => {
                body.value.walk(f);
            }

            _ => {}
        }
    }
}

impl DataTypeSetter for Statement {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        match self {
            Statement::If {
                condition,
                true_branch,
                else_if_branches,
                false_branch,
            } => {
                let mut out = condition.value._type_mut();
                out.append(&mut true_branch.value._type_mut());
                for else_if_branch in else_if_branches {
                    out.append(&mut else_if_branch.value.condition.value._type_mut());
                    out.append(&mut else_if_branch.value.body.value._type_mut());
                }
                if let Some(false_branch) = false_branch {
                    out.append(&mut false_branch.value._type_mut());
                }

                out
            }
            Statement::WhileLoop { condition, body } => {
                let mut out = condition.value._type_mut();
                out.append(&mut body.value._type_mut());
                out
            }
            Statement::VariableDecl { _type, value, .. } => {
                let mut out = vec![];
                if let Some(_type) = _type {
                    out.append(&mut _type.value._type_mut());
                }
                out.append(&mut value.value._type_mut());
                out
            }
            Statement::Return { value } => value.value._type_mut(),
            Statement::Expr(expr) => expr.value._type_mut(),
            Statement::VariableMutation {
                variable,
                new_value,
            } => {
                let mut out = variable.value._type_mut();
                out.append(&mut new_value.value._type_mut());
                out
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Spanned<Statement>>,
    pub variables: BTreeMap<String, Spanned<Variable>>,
    pub closure_params: Vec<Spanned<FunctionParam>>,
    pub generics: Vec<Spanned<DataType>>,
    pub return_type: DataType, // für anonyme funktion
    pub function_depth: usize, // tiefe innerhalb der funktion
}

impl Block {
    pub fn generics_set(&self) -> HashSet<DataType> {
        self.generics.iter().map(|g| g.value.clone()).collect()
    }

    pub fn return_statements(&self) -> Vec<Spanned<Statement>> {
        self.statements
            .iter()
            .filter(|s| {
                same_variant(
                    &s.value,
                    &Statement::Return {
                        value: unsafe { std::mem::zeroed() },
                    },
                )
            })
            .cloned()
            .collect_vec()
    }

    pub fn walk<F>(&self, f: &mut F)
    where
        F: FnMut(&Statement),
    {
        for statement in self.statements.iter() {
            f(&statement.value);
        }
    }

    pub fn get_variable(
        &self,
        name: &Spanned<String>,
    ) -> Result<Spanned<Variable>, Spanned<ParserError>> {
        match self.variables.get(&name.value) {
            Some(variable) => Ok(Spanned {
                value: variable.value.clone(),
                span: name.span,
            }),
            None => Err(Spanned {
                value: ParserError::VariableNotFound(name.value.clone()),
                span: name.span,
            }),
        }
    }
}

impl DataTypeSetter for Block {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        let mut out = self.return_type._type_mut();
        for statement in self.statements.iter_mut() {
            out.append(&mut statement.value._type_mut());
        }
        for local_variable in self.variables.values_mut() {
            out.append(&mut local_variable.value._type._type_mut());
        }
        out
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TypedExpr {
    pub expression: Expr,
    pub _type: DataType,
    pub raw: Option<String>, // wird nur von listen macro benutzt, sollte aber wahrschenlich einfach nur ein String sein
}

impl DataTypeSetter for TypedExpr {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        let mut out = self._type._type_mut();
        out.append(&mut self.expression._type_mut());
        out
    }
}

impl DataTypeGetter for TypedExpr {
    fn _type(&self) -> DataType {
        self._type.clone()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: Spanned<String>,
    pub is_mutable: bool,
    pub _type: DataType, // nicht spanned weil typ nicht angegeben sein muss
}

impl From<FunctionParam> for Variable {
    fn from(value: FunctionParam) -> Self {
        Variable {
            name: value.name.clone(),
            is_mutable: value.name.value == *CLASS_SELF_ARG_NAME,
            _type: value._type.value,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal),
    Variable(Variable),
    Binary {
        lhs: Box<Spanned<TypedExpr>>,
        op: Spanned<BinaryOperator>,
        rhs: Box<Spanned<TypedExpr>>,
    },
    Unary {
        op: Spanned<UnaryOperator>,
        expr: Box<Spanned<TypedExpr>>,
    },
    Cast {
        value: Box<Spanned<TypedExpr>>,
        to_type: Spanned<DataType>,
    },
    /// Referenz zu einem Objekt
    /// gibt einen Pointer auf das Objekt zurück, unterscheidet sich von dem cast
    /// weil der unterliegende pointer zurückgegeben wird und kein neuer erstellt wird
    /// ```txt
    /// | addresse | wert
    /// | 0000     | 1
    /// | 0001     | 2
    ///
    /// let x = 1 as *int;
    /// ~x // 2
    ///
    /// let x = &1;
    /// ~x // 1
    /// ```
    Reference {
        value: Box<Spanned<TypedExpr>>,
    },
    Call {
        function: Spanned<Function>,
        args: Spanned<Vec<Spanned<CallArg>>>,
    },
    Block {
        body: Spanned<Block>,
    },
    Index {
        base: Box<Spanned<TypedExpr>>,
        idx: Box<Spanned<TypedExpr>>,
    },
    FieldAccess {
        base: Box<Spanned<TypedExpr>>,
        field: Spanned<String>,
        field_idx: usize,
    },
    Deref(Box<Spanned<TypedExpr>>), // ~var

    ClassName(String), // statische methode: Foo::bar()
}

impl DataTypeSetter for Expr {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        match self {
            Expr::Literal(literal) => literal._type_mut(),
            Expr::Variable(variable) => variable._type._type_mut(),
            Expr::Binary { lhs, rhs, .. } => {
                let mut out = lhs.value._type_mut();
                out.append(&mut rhs.value._type_mut());
                out
            }
            Expr::Unary { expr, .. } => expr.value._type_mut(),
            Expr::Cast { value, to_type } => {
                let mut out = value.value._type_mut();
                out.append(&mut to_type.value._type_mut());
                out
            }
            Expr::Call { function, args } => {
                let mut out = vec![];
                out.push(DataTypeSettable::FunctionCall(
                    &mut function.value,
                    args.value
                        .clone()
                        .iter()
                        .map(|a| a.value.0._type.clone())
                        .collect_vec(),
                ));
                for arg in args.value.iter_mut() {
                    out.append(&mut arg.value.0._type_mut());
                }
                out
            }
            Expr::Index { base, .. } => base.value._type_mut(),
            Expr::FieldAccess { base, .. } => base.value._type_mut(),
            Expr::ClassName(_) => vec![],
            Expr::Deref(expr) => expr.value._type_mut(),
            Expr::Block { body } => {
                let mut out = body.value.return_type._type_mut();
                for statement in body.value.statements.iter_mut() {
                    out.append(&mut statement.value._type_mut());
                }
                out
            }
            Expr::Reference { value } => value.value._type_mut(),
        }
    }
}

pub const CLASS_SELF_ARG_NAME: &str = "self";

#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize, Hash)]
pub enum Trait {
    /// `+`
    #[display(fmt = "Add")]
    Add,
    /// `-`
    #[display(fmt = "Subtract")]
    Sub,
    /// `*`
    #[display(fmt = "Multiply")]
    Mul,
    /// `/`
    #[display(fmt = "Divide")]
    Div,
    /// `%`
    #[display(fmt = "Modulo")]
    Modulo,
    /// `[0]`
    #[display(fmt = "Index")]
    Index,
    /// `-1`
    #[display(fmt = "Negate")]
    Negate,
    /// `!`
    #[display(fmt = "BooleanNot")]
    BooleanNot,
    /// `==`
    #[display(fmt = "Equal")]
    Equal,
    /// `!=`
    #[display(fmt = "NotEqual")]
    NotEqual,
    /// `<`
    #[display(fmt = "LessThan")]
    LessThan,
    /// `<=`
    #[display(fmt = "LessThanOrEqual")]
    LessThanOrEqual,
    /// `>`
    #[display(fmt = "GreaterThan")]
    GreaterThan,
    /// `>=`
    #[display(fmt = "GreaterThanOrEqual")]
    GreaterThanOrEqual,
    /// `&&`
    #[display(fmt = "And")]
    And,
    /// `||`
    #[display(fmt = "Or")]
    Or,

    /// `as`
    #[display(fmt = "Cast")]
    Cast,
}

pub const TRAIT_NAMES_MAP: phf::Map<&str, Trait> = phf::phf_map!(
    "add" => Trait::Add,
    "sub" => Trait::Sub,
    "mul" => Trait::Mul,
    "div" => Trait::Div,
    "idx" => Trait::Index,
    "neg" => Trait::Negate,
    "not" => Trait::BooleanNot,

    "eq" => Trait::Equal,
    "ne" => Trait::NotEqual,
    "lt" => Trait::LessThan,
    "lte" => Trait::LessThanOrEqual,
    "gt" => Trait::GreaterThan,
    "gte" => Trait::GreaterThanOrEqual,
    "and" => Trait::And,
    "or" => Trait::Or,
    "mod" => Trait::Modulo,

    "as" => Trait::Cast,
);

fn get_default_implementation(type_: &DataType) -> Option<HashSet<TraitInfo>> {
    match type_ {
        DataType::Integer8 | DataType::Integer32 | DataType::Integer64 | DataType::Float => {
            Some(HashSet::from([
                (
                    Trait::Add,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    type_.to_owned(),
                ),
                (
                    Trait::Sub,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    type_.to_owned(),
                ),
                (
                    Trait::Mul,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    type_.to_owned(),
                ),
                (
                    Trait::Div,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    type_.to_owned(),
                ),
                (
                    Trait::Modulo,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    type_.to_owned(),
                ),
                (
                    Trait::Equal,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    DataType::Boolean,
                ),
                (
                    Trait::NotEqual,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    DataType::Boolean,
                ),
                (
                    Trait::LessThan,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    DataType::Boolean,
                ),
                (
                    Trait::LessThanOrEqual,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    DataType::Boolean,
                ),
                (
                    Trait::GreaterThan,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    DataType::Boolean,
                ),
                (
                    Trait::GreaterThanOrEqual,
                    vec![type_.to_owned(), type_.to_owned()],
                    None,
                    DataType::Boolean,
                ),
                (
                    Trait::Negate,
                    vec![type_.to_owned()],
                    None,
                    type_.to_owned(),
                ),
                (
                    Trait::Cast,
                    vec![type_.to_owned(), DataType::Integer8],
                    None,
                    DataType::Integer8,
                ),
                (
                    Trait::Cast,
                    vec![type_.to_owned(), DataType::Integer32],
                    None,
                    DataType::Integer32,
                ),
                (
                    Trait::Cast,
                    vec![type_.to_owned(), DataType::Integer64],
                    None,
                    DataType::Integer64,
                ),
            ]))
        }
        DataType::Boolean => Some(HashSet::from([
            (
                Trait::Equal,
                vec![type_.to_owned(), type_.to_owned()],
                None,
                DataType::Boolean,
            ),
            (
                Trait::NotEqual,
                vec![type_.to_owned(), type_.to_owned()],
                None,
                DataType::Boolean,
            ),
            (
                Trait::And,
                vec![type_.to_owned(), type_.to_owned()],
                None,
                DataType::Boolean,
            ),
            (
                Trait::Or,
                vec![type_.to_owned(), type_.to_owned()],
                None,
                DataType::Boolean,
            ),
            (
                Trait::BooleanNot,
                vec![type_.to_owned()],
                None,
                DataType::Boolean,
            ),
        ])),
        DataType::Array { value_type, .. } => Some(HashSet::from([(
            Trait::Index,
            vec![type_.to_owned(), DataType::Integer64],
            None,
            *value_type.clone(),
        )])),
        _ => None,
    }
}

impl Parser {
    pub(in crate::parser) fn check_trait_reqs(
        &mut self,
        _object: &DataType,
        trait_: &Trait,
        function: &Function,
    ) -> Option<ParserError> {
        let err = match trait_ {
            Trait::Index => {
                if !matches!(function.return_type.value, DataType::Pointer(_)) {
                    Some("Index function must return a pointer".to_string())
                } else {
                    None
                }
            }
            Trait::Add
            | Trait::Sub
            | Trait::Mul
            | Trait::Div
            | Trait::Modulo
            | Trait::Negate
            | Trait::BooleanNot
            | Trait::Equal
            | Trait::NotEqual
            | Trait::LessThan
            | Trait::LessThanOrEqual
            | Trait::GreaterThan
            | Trait::GreaterThanOrEqual
            | Trait::And
            | Trait::Or
            | Trait::Cast => None,
        };

        if let Some(err) = err {
            return Some(ParserError::TraitRequirementsNotFulfilled(err));
        }

        None
    }
}

// pub fn trait_requirements_fullfill) -> bool {
//     match trait_ {
//         Trait
//     }
// }

impl Trait {
    pub fn from_binary_operator(op: &BinaryOperator) -> Trait {
        match op {
            BinaryOperator::Add => Trait::Add,
            BinaryOperator::Subtract => Trait::Sub,
            BinaryOperator::Multiply => Trait::Mul,
            BinaryOperator::Divide => Trait::Div,
            BinaryOperator::Modulo => Trait::Modulo,
            BinaryOperator::Equal => Trait::Equal,
            BinaryOperator::NotEqual => Trait::NotEqual,
            BinaryOperator::LessThan => Trait::LessThan,
            BinaryOperator::LessThanOrEqual => Trait::LessThanOrEqual,
            BinaryOperator::GreaterThan => Trait::GreaterThan,
            BinaryOperator::GreaterThanOrEqual => Trait::GreaterThanOrEqual,
            BinaryOperator::And => Trait::And,
            BinaryOperator::Or => Trait::Or,
        }
    }

    pub fn from_unary_operator(op: &UnaryOperator) -> Option<Trait> {
        match op {
            UnaryOperator::Minus => Some(Trait::Sub),
            UnaryOperator::Not => Some(Trait::Negate),
        }
    }

    pub fn method_name(&self) -> String {
        for (name, trait_) in TRAIT_NAMES_MAP.entries() {
            if trait_ == self {
                return name.to_string();
            }
        }

        unreachable!()
    }
    /// anzahl der parameter inklusive self
    pub fn param_len(&self) -> usize {
        match self {
            Trait::Add
            | Trait::Sub
            | Trait::Mul
            | Trait::Div
            | Trait::Modulo
            | Trait::Equal
            | Trait::NotEqual
            | Trait::LessThan
            | Trait::LessThanOrEqual
            | Trait::GreaterThan
            | Trait::GreaterThanOrEqual
            | Trait::And
            | Trait::Or
            | Trait::Index
            | Trait::Cast => 1,
            Trait::Negate | Trait::BooleanNot => 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CallArg(pub TypedExpr);

#[derive(
    Debug, Display, Default, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Hash,
)]
pub enum DataType {
    #[display(fmt = "int8")]
    #[strum(serialize = "int8")]
    Integer8,

    #[display(fmt = "int16")]
    #[strum(serialize = "int16")]
    Integer16,

    #[display(fmt = "int32")]
    #[strum(serialize = "int32")]
    Integer32,

    #[display(fmt = "int64")]
    #[strum(serialize = "int64")]
    Integer64,

    #[display(fmt = "float")]
    #[strum(serialize = "float")]
    Float,

    #[display(fmt = "bool")]
    #[strum(serialize = "bool")]
    Boolean,

    #[display(fmt = "*{}", _0)]
    Pointer(Box<DataType>),

    #[display(fmt = "{}", _0)]
    Custom(CustomDataType),

    #[display(fmt = "[{}; {}]", value_type, len)]
    Array {
        value_type: Box<DataType>,
        len: usize,
    },
    #[display(fmt = "{}", _0)]
    Generic(String),

    #[display(fmt = "void")]
    #[strum(serialize = "void")]
    #[default]
    None,

    // #[display(fmt = "any")]
    // Any,
    #[display(fmt = "type")]
    DataType,
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::Integer8 => 1,
            DataType::Integer16 => 2,
            DataType::Integer32 => 4,
            DataType::Integer64 => 8,
            DataType::Float => 4,
            DataType::Boolean => 1,
            DataType::Pointer(_) => size_of::<usize>(),
            DataType::Custom(inner) => inner
                .fields
                .value
                .iter()
                .map(|f| f._type.value.size())
                .sum(),
            DataType::Array { value_type, len } => value_type.size() * len,
            DataType::Generic(_) => 8,
            DataType::None => 0,
            DataType::DataType => 8,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTypeInfo {
    pub methods: Vec<String>,
    pub parent_type: DataType,
    // TODO: hashmap?
    pub traits: HashSet<TraitInfo>,
    // pub export: bool,
}

impl DataTypeInfo {
    pub fn implements_trait(&self, trait_: &Trait, params: &[DataType]) -> bool {
        if let Some(traits) = get_default_implementation(&self.parent_type) {
            // standard implementierungen, float, int, bool
            if let Some((_, _, _, _)) = traits
                .iter()
                .find(|(t, p, _, _)| t == trait_ && p == params)
            {
                return true;
            }
        }

        self.traits
            .iter()
            .any(|(t, p, _, _)| t == trait_ && p == params)
    }

    pub fn get_trait_override_function_name(
        &self,
        trait_: &Trait,
        params: &[DataType],
    ) -> Option<String> {
        self.traits.iter().find_map(|(t, p, name, _)| {
            if t == trait_ && p == params {
                name.clone()
            } else {
                None
            }
        })
    }

    pub fn get_trait_return_type(&self, trait_: &Trait, params: &[DataType]) -> Option<DataType> {
        for (t, p, _, return_type) in self.traits.iter() {
            if t == trait_ && p == params {
                return Some(return_type.clone());
            }
        }

        if let Some(traits) = get_default_implementation(&self.parent_type) {
            if let Some((_, _, _, return_type)) = traits
                .iter()
                .find(|(t, p, _, _)| t == trait_ && p == params)
            {
                return Some(return_type.clone());
            }
        }
        // TODO: Option<DataType> ?
        None
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

impl BinaryOperator {
    pub fn is_ordering(&self) -> bool {
        matches!(
            self,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::LessThan
                | BinaryOperator::LessThanOrEqual
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterThanOrEqual
        )
    }

    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Add | BinaryOperator::Subtract => 3,
            BinaryOperator::Multiply | BinaryOperator::Divide => 4,
            // BinaryOperator::Modulo => 4,
            BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Modulo => 2,
            BinaryOperator::And | BinaryOperator::Or => 1,
        }
    }
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Hash)]
pub enum UnaryOperator {
    Not,
    Minus,
}

impl UnaryOperator {}

// impl PartialEq for DataType {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (DataType::Integer8, DataType::Integer8) => true,
//             (DataType::Integer32, DataType::Integer32) => true,
//             (DataType::Integer64, DataType::Integer64) => true,
//             (DataType::Float, DataType::Float) => true,
//             (DataType::Boolean, DataType::Boolean) => true,
//             (DataType::Pointer(inner1), DataType::Pointer(inner2)) => inner1 == inner2,
//             (DataType::Custom(inner1), DataType::Custom(inner2)) => inner1 == inner2,
//             (
//                 DataType::Array {
//                     value_type: inner1,
//                     len: len1,
//                 },
//                 DataType::Array {
//                     value_type: inner2,
//                     len: len2,
//                 },
//             ) => inner1 == inner2 && len1 == len2,
//             (DataType::None, DataType::None) => true,
//             (DataType::Generic(inner1), DataType::Generic(inner2)) => inner1 == inner2,
//             (DataType::DataType, DataType::DataType) => true,
//             (DataType::DataType, DataType::Generic(_)) => true,
//             _ => false,
//         }
//     }
// }

impl InternalNameGetter for DataType {
    fn internal_name(&self) -> String {
        match self {
            DataType::Integer8
            | DataType::Integer16
            | DataType::Integer32
            | DataType::Integer64
            | DataType::Float
            | DataType::Boolean
            | DataType::Array { .. }
            | DataType::Generic(..)
            | DataType::DataType
            | DataType::None => self.to_string(),
            DataType::Pointer(inner) => format!("ptr-{}", inner.internal_name()),
            DataType::Custom(custom_type) => custom_type.name.clone(),
        }
    }
}

impl DataType {
    pub fn can_be_converted_to(&self, to: &DataType) -> bool {
        matches!(
            (self, to),
            (
                DataType::Integer8
                    | DataType::Integer16
                    | DataType::Integer32
                    | DataType::Integer64,
                DataType::Pointer(_)
            ) | (DataType::Pointer(_), DataType::Pointer(_))
                | (DataType::Pointer(_), DataType::Integer64,)
                | (
                    DataType::Integer8
                        | DataType::Integer16
                        | DataType::Integer32
                        | DataType::Integer64
                        | DataType::Boolean,
                    DataType::Integer8
                        | DataType::Integer16
                        | DataType::Integer32
                        | DataType::Integer64
                        | DataType::Boolean,
                )
                | (
                    DataType::Integer8
                        | DataType::Integer16
                        | DataType::Integer32
                        | DataType::Integer64,
                    DataType::Float,
                )
                | (
                    DataType::Float,
                    DataType::Integer8
                        | DataType::Integer16
                        | DataType::Integer32
                        | DataType::Integer64,
                )
        )

        // (
        //     DataType::Integer8 | DataType::Integer64,
        //     DataType::Integer32 | DataType::Integer64,
        // ) => false,
        // (DataType::Integer32, DataType::Integer8) => true,
        // (DataType::Float, _) => false,
        // (DataType::Boolean, _) => false,
        // (DataType::Pointer(_), _) => false,
        // (DataType::Custom(from), DataType::Custom(to)) => {
        //     from.subtype_of == Some(to.name.clone())
        // }
        // (DataType::Array { .. }, _) => todo!(),
        // (DataType::None, _) => todo!(),
        // (DataType::DataType, _) => todo!(),
        // // TODO: ?
        // (DataType::Integer64, DataType::Integer32 | DataType::Integer8) => true,
        // (DataType::Generic(_), _) => unreachable!(),
        // _ => false,
        // DataType::Integer8 => false,
        // DataType::Integer32 => to == &DataType::Integer8,
        // DataType::Float => false,
        // DataType::Boolean => false,
        // DataType::Pointer(..) => false,
        // DataType::Custom(..) => todo!(),
        // DataType::Array { .. } => todo!(),
        // DataType::None => todo!(),
        // DataType::DataType => todo!(),
        // DataType::Integer64 => to == &DataType::Integer32 || to == &DataType::Integer8,
        // DataType::Generic(_) => unreachable!(),
        // }
    }

    pub fn get_integer_type() -> DataType {
        match std::mem::size_of::<isize>() {
            8 => DataType::Integer64,
            4 => DataType::Integer32,
            _ => unreachable!(),
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            DataType::Integer8 | DataType::Integer16 | DataType::Integer32 | DataType::Integer64
        )
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, DataType::Boolean)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, DataType::Float)
    }
}

impl CommonGeneric for DataType {
    fn is_generic(&self) -> bool {
        matches!(self, DataType::Generic(_))
            || matches!(self, DataType::Custom(inner) if inner.is_generic())
            || matches!(self, DataType::Pointer(inner) if inner.is_generic())
    }

    fn generics(&self) -> OrderSet<String> {
        match self {
            DataType::Generic(name) => {
                let mut out = OrderSet::new();
                out.insert(name.clone());
                out
            }
            DataType::Custom(inner) => inner.generics.iter().map(|g| g.value.to_string()).collect(),
            DataType::Pointer(inner) => inner.generics(),
            _ => OrderSet::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CustomDataType {
    pub display_name: String,
    pub name: String,
    pub fields: Spanned<Vec<FunctionParam>>,
    // pub methods: Vec<String>,
    pub subtypes: HashMap<Vec<DataType>, CustomDataType>,
    pub generics: Vec<Spanned<DataType>>,
    pub subtype_of: Option<String>,
    // ```
    // struct Foo<T> {
    //     bar: int,
    // }
    pub is_generic: bool, // falls es einen generischen Paremeter gibt, der aber nicht verwendet wird
}

impl Hash for CustomDataType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.display_name.hash(state);
        self.name.hash(state);
        self.fields.value.hash(state);
    }
}

impl Display for CustomDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // if !self.generics.is_empty() {
        //     write!(
        //         f,
        //         "{}<{}>",
        //         self.display_name,
        //         self.generics
        //             .iter()
        //             .map(|g| g.value.to_string())
        //             .collect::<Vec<_>>()
        //             .join(", ")
        //     )
        // } else {
        write!(f, "{}", self.display_name)
        // }
    }
}

impl PartialEq for CustomDataType {
    fn eq(&self, other: &Self) -> bool {
        self.display_name == other.display_name && self.fields.value == other.fields.value
    }
}

impl Eq for CustomDataType {}

impl CustomDataType {
    pub fn generics_set(&self) -> HashSet<DataType> {
        self.generics.iter().map(|g| g.value.clone()).collect()
    }

    pub fn subtype(
        &self,
        specific_types: &[DataType],
        program: &mut Program,
        subtype_traits: bool,
    ) -> CustomDataType {
        let generic_map: HashMap<String, DataType> = self
            .generics
            .iter()
            .map(|g| {
                if let DataType::Generic(name) = &g.value {
                    name.to_owned()
                } else {
                    unreachable!()
                }
            })
            .zip(specific_types.iter().cloned())
            .collect();

        let mut fields = self.fields.clone();

        let mut field_types: Vec<DataTypeSettable> = {
            let mut out = vec![];
            for field in fields.value.iter_mut() {
                out.append(&mut field._type.value._type_mut());
            }

            out
        };

        specify_generics(&mut field_types, &generic_map, program, true);

        let display_name = format!(
            "{}<{}>",
            self.display_name,
            generic_map
                .iter()
                .map(|(_, s)| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let name = format!(
            "{}--{}",
            self.name,
            generic_map
                .iter()
                .map(|(_, s)| s.internal_name())
                .collect::<Vec<_>>()
                .join(".")
        );

        // if let

        if let Some(Spanned {
            value: DataType::Custom(inner),
            ..
        }) = program.custom_types.get(&name)
        {
            // panic!("class already exists: {:?}", inner.methods.keys());
            // panic!("fef");
            return inner.clone();
        }

        // let mut generic_subtypes = AHashMap::new();

        // for (generic, specific) in generic_map.iter() {
        //     if let DataType::Custom(custom_type) = specific {
        //         let new_subtype = custom_type.subtype(specific_types, parser);
        //         generic_subtypes.insert(generic.clone(), new_subtype);
        //     }
        // }

        // println!("generic_subtypes: {:?}", generic_subtypes);

        let out = CustomDataType {
            display_name,
            name,
            fields,
            // methods: self.methods.clone(),
            subtypes: HashMap::new(),
            generics: vec![],
            subtype_of: Some(self.name.clone()),
            is_generic: false,
        };

        if out.is_generic() {
            return self.clone();
        }

        let data_type_info = program.get_type_info(&DataType::Custom(self.clone()));

        let mut new_data_type_info = DataTypeInfo {
            methods: data_type_info.methods.clone(),
            parent_type: DataType::Custom(out.clone()),
            traits: HashSet::new(),
        };

        if subtype_traits {
            for trait_ in data_type_info.traits {
                let mut trait_: TraitInfo = trait_;
                let mut types: Vec<DataTypeSettable> = vec![];
                // trait_.1[0] ist der subtyp, muss hier gesetzt werden
                trait_.1[0] = DataType::Custom(out.clone());

                types.append(
                    &mut trait_
                        .1
                        .iter_mut()
                        .flat_map(|t| t._type_mut())
                        .collect_vec(),
                );
                types.append(&mut trait_.3._type_mut());

                specify_generics(&mut types, &generic_map, program, false);

                let trait_name = trait_.clone().2.unwrap();

                let trait_function = program.functions.get(&trait_name).unwrap().clone();
                let subtype = trait_function
                    .value
                    .subtype(&generic_map, None, program, false);

                if let Some(ref mut name) = trait_.2 {
                    *name = subtype.name.clone();
                }

                program.functions.insert(
                    subtype.name.clone(),
                    Spanned {
                        value: subtype,
                        span: Span::default(),
                    },
                );

                new_data_type_info.traits.insert(trait_);
            }

            program
                .data_types
                .insert(out.name.clone(), new_data_type_info);
        }

        // parent = get_parent(parser);
        // let mut methods = self.methods.clone();

        // out.methods = methods;

        // println!("name: {}; methods_num: {}", out.name, out.methods.len());
        program.custom_types.insert(
            out.name.clone(),
            Spanned {
                value: DataType::Custom(out.clone()),
                span: Span::default(),
            },
        );
        out
    }

    // pub fn refresh_subtype_methods(&mut self) {
    //     for (_, subtype) in self.subtypes.iter_mut() {
    //         for method in self.methods.iter() {
    //             if !subtype.methods.contains(method) {
    //                 subtype.methods.push(method.clone());
    //             }
    //         }
    //     }
    // }
}

impl CommonGeneric for CustomDataType {
    fn is_generic(&self) -> bool {
        self.is_generic || self.fields.value.iter().any(|f| f._type.value.is_generic())
    }

    fn generics(&self) -> OrderSet<String> {
        let mut out = OrderSet::new();
        for field in self.fields.value.iter() {
            let sub: OrderSet<String> = field._type.value.generics();
            out.extend(sub);
        }

        out
    }
}

impl DataTypeSetter for CustomDataType {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        let mut out = vec![];
        for field in self.fields.value.iter_mut() {
            out.append(&mut field._type.value._type_mut());
        }

        for generic in self.generics.iter_mut() {
            out.append(&mut generic.value._type_mut());
        }

        out
    }
}

impl DataTypeGetterRecursive for CustomDataType {
    fn types(&self) -> Vec<DataType> {
        let mut out = vec![];
        for field in self.fields.value.iter() {
            out.append(&mut field._type.value.types());
        }

        for generic in self.generics.iter() {
            out.append(&mut generic.value.types());
        }

        out
    }
}

impl DataTypeSetter for DataType {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        match self {
            DataType::Integer8
            | DataType::Integer16
            | DataType::Integer32
            | DataType::Integer64
            | DataType::Float
            | DataType::Boolean
            | DataType::None
            | DataType::DataType => vec![],
            DataType::Pointer(inner) => inner._type_mut(),
            DataType::Custom(_inner) => vec![DataTypeSettable::DataType(self)],
            DataType::Array { value_type, .. } => value_type._type_mut(),
            DataType::Generic(_inner) => vec![DataTypeSettable::DataType(self)],
        }
    }
}

impl DataTypeGetterRecursive for DataType {
    fn types(&self) -> Vec<DataType> {
        match self {
            DataType::Pointer(inner) => inner.types(),
            DataType::Custom(inner) => inner.types(),
            DataType::Array { value_type, .. } => value_type.types(),
            DataType::Generic(_inner) => vec![self.clone()],
            _ => vec![self.clone()],
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ClassLiteral {
    pub _type: DataType,
    pub fields: Spanned<Vec<(Spanned<String>, Spanned<TypedExpr>)>>,
}

impl DataTypeSetter for ClassLiteral {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        let mut out = self._type._type_mut();
        for (_, expr) in self.fields.value.iter_mut() {
            out.append(&mut expr.value._type_mut());
        }
        out
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayLiteral {
    pub value_type: DataType,
    pub values: Spanned<Vec<Spanned<TypedExpr>>>,
}

impl DataTypeGetter for ArrayLiteral {
    fn _type(&self) -> DataType {
        DataType::Array {
            value_type: Box::new(self.value_type.clone()),
            len: self.values.value.len(),
        }
    }
}

impl DataTypeSetter for ArrayLiteral {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable> {
        let mut out = self.value_type._type_mut();
        for value in self.values.value.iter_mut() {
            out.append(&mut value.value._type_mut());
        }
        out
    }
}

// impl Operator {
//     pub fn precedence(&self) -> u8 {
//         match self {
//             Operator::Add | Operator::Subtract | Operator::And => 2,
//             Operator::Multiply | Operator::Divide | Operator::Or => 3,
//             Operator::Exponentiate => 4,
//             Operator::LessThan
//             | Operator::LessThanOrEqual
//             | Operator::GreaterThan
//             | Operator::GreaterThanOrEqual
//             | Operator::Equal
//             | Operator::NotEqual => 1,
//         }
//     }

//     pub fn is_ordering(&self) -> bool {
//         matches!(
//             self,
//             Operator::Equal
//                 | Operator::NotEqual
//                 | Operator::LessThan
//                 | Operator::LessThanOrEqual
//                 | Operator::GreaterThan
//                 | Operator::GreaterThanOrEqual
//         )
//     }

// pub fn is_unary(&self) -> bool {
//     matches!(
//         self,
//         Operator::Subtract,
//     )
// }

// pub fn is_logical(&self) -> bool {
//     todo!()
// }

// #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Copy)]
// pub enum UnaryOperator {
//     Plus,
//     Minus,
//     Not,
// }

pub trait DataTypeGetter {
    fn _type(&self) -> DataType;
}

pub trait DataTypeGetterRecursive {
    fn types(&self) -> Vec<DataType>;
}

#[derive(Debug)]
pub enum DataTypeSettable<'a> {
    DataType(&'a mut DataType),
    FunctionCall(&'a mut Function, Vec<DataType>),
}

pub trait DataTypeSetter {
    fn _type_mut(&mut self) -> Vec<DataTypeSettable>;
}

pub trait CommonGeneric {
    fn is_generic(&self) -> bool;
    fn generics(&self) -> OrderSet<String>;
}

pub trait SubTypeBuilder {
    fn subtype(&self, specific_types: &[DataType]) -> Self;
}

pub trait InternalNameGetter {
    fn internal_name(&self) -> String;
}
