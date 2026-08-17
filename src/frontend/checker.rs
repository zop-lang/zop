//! Module declarations and source-type resolution.
//!
//! This pass builds stable function signatures before checking any function body.

use std::collections::{HashMap, HashSet};

use crate::{
    diagnostic::{Diagnostic, Diagnostics, Stage},
    hir,
    source::Span,
    syntax::{self, parse},
};

use super::function::FunctionContext;

/// Parse and type-check one source file.
pub fn analyze(source: &str) -> Result<hir::Module, Diagnostics> {
    let syntax = parse(source)?;
    Checker::new(&syntax).check()
}

#[derive(Clone)]
pub(super) struct Signature {
    pub(super) id: hir::FunctionId,
    pub(super) kind: hir::FunctionKind,
    pub(super) name: String,
    pub(super) parameters: Vec<SignatureParameter>,
    pub(super) result: hir::Type,
}

#[derive(Clone)]
pub(super) struct SignatureParameter {
    pub(super) mode: hir::ParameterMode,
    pub(super) name: String,
    pub(super) ty: hir::Type,
    pub(super) span: Span,
}

pub(super) struct Checker<'syntax> {
    syntax: &'syntax syntax::Module,
    signatures: Vec<Signature>,
    pub(super) functions: HashMap<String, hir::FunctionId>,
    errors: Diagnostics,
}

impl<'syntax> Checker<'syntax> {
    fn new(syntax: &'syntax syntax::Module) -> Self {
        let mut checker =
            Self { syntax, signatures: Vec::new(), functions: HashMap::new(), errors: Vec::new() };
        checker.declare_functions();
        checker
    }

    fn check(mut self) -> Result<hir::Module, Diagnostics> {
        let mut functions = Vec::new();
        for (index, function) in self.syntax.functions.iter().enumerate() {
            let signature = self.signatures[index].clone();
            let mut context = FunctionContext::new(&mut self, signature);
            if let Some(function) = context.check(function) {
                functions.push(function);
            }
        }

        if self.errors.is_empty() { Ok(hir::Module { functions }) } else { Err(self.errors) }
    }

    fn declare_functions(&mut self) {
        for (index, function) in self.syntax.functions.iter().enumerate() {
            let id = hir::FunctionId(index);
            if self.functions.insert(function.name.clone(), id).is_some() {
                self.error("S0003", "function is declared more than once", function.span);
            }
            if function.failure != syntax::FailureType::None {
                self.error(
                    "S0001",
                    "error channels are not in the scalar frontend slice",
                    function.span,
                );
            }
            let mut parameter_names = HashSet::new();
            for parameter in &function.parameters {
                if !parameter_names.insert(&parameter.name) {
                    self.error("S0003", "parameter is declared more than once", parameter.span);
                }
            }

            let parameters = function
                .parameters
                .iter()
                .map(|parameter| self.signature_parameter(parameter))
                .collect();
            let result =
                function.return_type.as_ref().map_or(hir::Type::Unit, |ty| self.resolve_type(ty));
            self.signatures.push(Signature {
                id,
                kind: function_kind(function.kind),
                name: function.name.clone(),
                parameters,
                result,
            });
        }
    }

    fn signature_parameter(&mut self, parameter: &syntax::Parameter) -> SignatureParameter {
        if parameter.mode != syntax::ParameterMode::Borrow {
            self.error(
                "S0001",
                "parameter ownership modes are not in the scalar frontend slice",
                parameter.span,
            );
        }
        SignatureParameter {
            mode: parameter_mode(parameter.mode),
            name: parameter.name.clone(),
            ty: self.resolve_type(&parameter.ty),
            span: parameter.span,
        }
    }

    fn resolve_type(&mut self, ty: &syntax::TypeExpression) -> hir::Type {
        if !ty.dimensions.is_empty() {
            self.error("S0001", "tensor types are not in the scalar frontend slice", ty.span);
            return hir::Type::Unit;
        }

        match ty.name.as_str() {
            "i32" => hir::Type::I32,
            "i64" => hir::Type::I64,
            "f32" => hir::Type::F32,
            "f64" => hir::Type::F64,
            "bool" => hir::Type::Bool,
            "str" => hir::Type::String,
            _ => {
                self.error("S0001", format!("unknown type `{}`", ty.name), ty.span);
                hir::Type::Unit
            }
        }
    }

    pub(super) fn signature(&self, id: hir::FunctionId) -> Signature {
        self.signatures[id.0].clone()
    }

    pub(super) fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.errors.push(Diagnostic::new(Stage::Semantic, code, message, span));
    }
}

fn function_kind(kind: syntax::FunctionKind) -> hir::FunctionKind {
    match kind {
        syntax::FunctionKind::Host => hir::FunctionKind::Host,
        syntax::FunctionKind::Kernel => hir::FunctionKind::Kernel,
    }
}

fn parameter_mode(mode: syntax::ParameterMode) -> hir::ParameterMode {
    match mode {
        syntax::ParameterMode::Borrow => hir::ParameterMode::Borrow,
        syntax::ParameterMode::Mut => hir::ParameterMode::Mut,
        syntax::ParameterMode::Give => hir::ParameterMode::Give,
    }
}
