// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Module declarations and source-type resolution.
//!
//! This pass builds stable function signatures before checking any function body.
//!
//! Flow:
//!
//! 1. Index every module function name and assign a stable identity.
//! 2. Resolve every signature without inspecting function bodies.
//! 3. Mark recursive call-graph components and require explicit result types.
//! 4. Check each body against its resolved signature and emit typed HIR.

use std::collections::{HashMap, HashSet};

use crate::{
    diagnostic::{Diagnostic, Diagnostics, Stage},
    hir,
    source::Span,
    syntax::{self, parse},
};

use super::function::FunctionContext;

/// Parse and type-check one source file.
///
/// # Errors
///
/// Returns lexical, parse, or semantic diagnostics. No HIR escapes when any
/// compiler phase rejects the source.
pub fn analyze(source: &str) -> Result<hir::Module, Diagnostics> {
    let syntax = parse(source)?;
    Checker::new(&syntax).check()
}

/// Resolved callable interface available before any body is checked.
#[derive(Clone)]
pub(super) struct Signature {
    /// Stable module identity assigned during name indexing.
    pub(super) id: hir::FunctionId,

    /// Host or device declaration kind.
    pub(super) kind: hir::FunctionKind,

    /// Source name retained for diagnostics and emitted symbols.
    pub(super) name: String,

    /// Resolved parameters in calling-convention order.
    pub(super) parameters: Vec<SignatureParameter>,

    /// Explicit or implicit unit result type.
    pub(super) result: hir::Type,
}

/// Resolved parameter contract used by calls and body-local bindings.
#[derive(Clone)]
pub(super) struct SignatureParameter {
    /// Checked ownership access mode.
    pub(super) mode: hir::ParameterMode,

    /// Label used by named arguments and the body binding.
    pub(super) name: String,

    /// Concrete scalar type accepted by the bootstrap.
    pub(super) ty: hir::Type,

    /// Source range covering the declaration.
    pub(super) span: Span,
}

/// Module semantic pass with explicit name, signature, and body phases.
pub(super) struct Checker<'syntax> {
    /// Parsed module whose source structure remains immutable during checking.
    syntax: &'syntax syntax::Module,

    /// Resolved signatures indexed by [`hir::FunctionId`].
    signatures: Vec<Signature>,

    /// Module function names mapped to stable identities before body checking.
    pub(super) functions: HashMap<String, hir::FunctionId>,

    /// Semantic diagnostics accumulated across all phases.
    errors: Diagnostics,
}

impl<'syntax> Checker<'syntax> {
    fn new(syntax: &'syntax syntax::Module) -> Self {
        Self { syntax, signatures: Vec::new(), functions: HashMap::new(), errors: Vec::new() }
    }

    fn check(mut self) -> Result<hir::Module, Diagnostics> {
        self.index_functions();
        self.resolve_signatures();
        self.require_explicit_recursive_results();

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

    fn index_functions(&mut self) {
        for (index, function) in self.syntax.functions.iter().enumerate() {
            let id = hir::FunctionId(index);
            if self.functions.insert(function.name.clone(), id).is_some() {
                self.error("S0003", "function is declared more than once", function.span);
            }
        }
    }

    fn resolve_signatures(&mut self) {
        for (index, function) in self.syntax.functions.iter().enumerate() {
            let id = hir::FunctionId(index);
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

    fn require_explicit_recursive_results(&mut self) {
        let call_graph = self.call_graph();
        let recursive = recursive_functions(&call_graph);
        for (index, function) in self.syntax.functions.iter().enumerate() {
            if recursive[index] && function.return_type.is_none() {
                self.error(
                    "S0010",
                    "recursive function requires an explicit result type",
                    function.span,
                );
            }
        }
    }

    fn call_graph(&self) -> Vec<Vec<hir::FunctionId>> {
        self.syntax
            .functions
            .iter()
            .map(|function| {
                let mut callees = Vec::new();
                collect_callees(&function.body, &self.functions, &mut callees);
                callees.sort_unstable_by_key(|id| id.0);
                callees.dedup();
                callees
            })
            .collect()
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

    /// Clone one fully resolved signature by stable identity.
    pub(super) fn signature(&self, id: hir::FunctionId) -> Signature {
        self.signatures[id.0].clone()
    }

    /// Record one semantic diagnostic without abandoning sibling checks.
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

fn collect_callees(
    block: &syntax::Block,
    functions: &HashMap<String, hir::FunctionId>,
    callees: &mut Vec<hir::FunctionId>,
) {
    for expression in &block.expressions {
        collect_expression_callees(expression, functions, callees);
    }
}

fn collect_expression_callees(
    expression: &syntax::Expression,
    functions: &HashMap<String, hir::FunctionId>,
    callees: &mut Vec<hir::FunctionId>,
) {
    match &expression.kind {
        syntax::ExpressionKind::Assign { target, value }
        | syntax::ExpressionKind::Binary { left: target, right: value, .. } => {
            collect_expression_callees(target, functions, callees);
            collect_expression_callees(value, functions, callees);
        }
        syntax::ExpressionKind::Unary { operand, .. }
        | syntax::ExpressionKind::Member { object: operand, .. }
        | syntax::ExpressionKind::Fail(operand)
        | syntax::ExpressionKind::Try(operand) => {
            collect_expression_callees(operand, functions, callees);
        }
        syntax::ExpressionKind::Call { callee, arguments } => {
            if let syntax::ExpressionKind::Name(name) = &callee.kind
                && let Some(id) = functions.get(name)
            {
                callees.push(*id);
            }
            collect_expression_callees(callee, functions, callees);
            for argument in arguments {
                collect_expression_callees(&argument.value, functions, callees);
            }
        }
        syntax::ExpressionKind::If { condition, then_block, else_block } => {
            collect_expression_callees(condition, functions, callees);
            collect_callees(then_block, functions, callees);
            if let Some(else_block) = else_block {
                collect_callees(else_block, functions, callees);
            }
        }
        syntax::ExpressionKind::Return(value) => {
            if let Some(value) = value {
                collect_expression_callees(value, functions, callees);
            }
        }
        syntax::ExpressionKind::Catch { expression, arms } => {
            collect_expression_callees(expression, functions, callees);
            for arm in arms {
                collect_callees(&arm.body, functions, callees);
            }
        }
        syntax::ExpressionKind::Name(_)
        | syntax::ExpressionKind::Integer(_)
        | syntax::ExpressionKind::Float(_)
        | syntax::ExpressionKind::Bool(_)
        | syntax::ExpressionKind::String(_) => {}
    }
}

/// Mark strongly connected call-graph members without recursing on user input.
///
/// The first iterative pass records depth-first finish order. The second walks
/// reversed edges in that order. Members reached together form one recursive
/// component. An iterative implementation prevents deeply nested source from
/// overflowing the compiler stack.
fn recursive_functions(graph: &[Vec<hir::FunctionId>]) -> Vec<bool> {
    let mut visited = vec![false; graph.len()];
    let mut finish_order = Vec::with_capacity(graph.len());
    for start in 0..graph.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![(start, 0)];
        while let Some((node, edge_index)) = stack.last_mut() {
            if let Some(next) = graph[*node].get(*edge_index) {
                *edge_index += 1;
                if !visited[next.0] {
                    visited[next.0] = true;
                    stack.push((next.0, 0));
                }
            } else {
                finish_order.push(*node);
                stack.pop();
            }
        }
    }

    let mut reverse = vec![Vec::new(); graph.len()];
    for (source, targets) in graph.iter().enumerate() {
        for target in targets {
            reverse[target.0].push(source);
        }
    }

    let mut component = vec![usize::MAX; graph.len()];
    let mut component_sizes = Vec::new();
    for start in finish_order.into_iter().rev() {
        if component[start] != usize::MAX {
            continue;
        }
        let component_id = component_sizes.len();
        let mut size = 0;
        let mut stack = vec![start];
        component[start] = component_id;
        while let Some(node) = stack.pop() {
            size += 1;
            for predecessor in &reverse[node] {
                if component[*predecessor] == usize::MAX {
                    component[*predecessor] = component_id;
                    stack.push(*predecessor);
                }
            }
        }
        component_sizes.push(size);
    }

    graph
        .iter()
        .enumerate()
        .map(|(function, callees)| {
            component_sizes[component[function]] > 1
                || callees.iter().any(|callee| callee.0 == function)
        })
        .collect()
}
