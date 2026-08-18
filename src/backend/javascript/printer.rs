//! Deterministic JavaScript printer with precedence-aware parentheses.

use std::fmt::Write;

use super::ast::{self, BinaryOperator as B, Expression as E, Statement as S};

pub(super) fn print(module: &ast::Module) -> String {
    let mut output = String::new();
    for (index, function) in module.functions.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        print_function(&mut output, function);
    }
    if !module.functions.is_empty() {
        output.push('\n');
    }
    print_exports(&mut output, &module.exports);
    output
}

fn print_function(output: &mut String, function: &ast::Function) {
    write!(output, "function {}(", function.name).expect("writing to String cannot fail");
    write_joined(output, &function.parameters, |output, parameter| output.push_str(parameter));
    output.push_str(") {\n");
    for statement in &function.body {
        output.push_str("    ");
        print_statement(output, statement);
        output.push('\n');
    }
    output.push_str("}\n");
}

fn print_statement(output: &mut String, statement: &ast::Statement) {
    match statement {
        S::Let { name, value } => {
            write!(output, "let {name} = ").expect("writing to String cannot fail");
            print_expression(output, value, 0);
        }
        S::Assign { name, value } => {
            write!(output, "{name} = ").expect("writing to String cannot fail");
            print_expression(output, value, 0);
        }
        S::Expression(expression) => print_expression(output, expression, 0),
        S::Return(Some(expression)) => {
            output.push_str("return ");
            print_expression(output, expression, 0);
        }
        S::Return(None) => output.push_str("return"),
    }
    output.push(';');
}

fn print_expression(output: &mut String, expression: &ast::Expression, parent: u8) {
    let precedence = precedence(expression);
    let parenthesize = precedence < parent;
    if parenthesize {
        output.push('(');
    }
    match expression {
        E::Identifier(name) | E::Number(name) => output.push_str(name),
        E::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        E::String(value) => print_string(output, value),
        E::Call { function, arguments } => {
            output.push_str(function);
            output.push('(');
            write_joined(output, arguments, |output, argument| {
                print_expression(output, argument, 0);
            });
            output.push(')');
        }
        E::Unary { operator, operand } => {
            output.push_str(operator.text());
            if operator.fuses_with(operand) {
                output.push('(');
                print_expression(output, operand, 0);
                output.push(')');
            } else {
                print_expression(output, operand, precedence + 1);
            }
        }
        E::Binary { operator, left, right } => {
            print_expression(output, left, precedence);
            write!(output, " {} ", operator.text()).expect("writing to String cannot fail");
            print_expression(output, right, precedence + 1);
        }
    }
    if parenthesize {
        output.push(')');
    }
}

fn print_exports(output: &mut String, exports: &[ast::Export]) {
    if exports.is_empty() {
        output.push_str("export {};\n");
        return;
    }
    output.push_str("export { ");
    write_joined(output, exports, |output, export| {
        write!(output, "{} as {}", export.local, export.exported)
            .expect("writing to String cannot fail");
    });
    output.push_str(" };\n");
}

fn write_joined<T>(
    output: &mut String,
    values: &[T],
    mut write_value: impl FnMut(&mut String, &T),
) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_value(output, value);
    }
}

fn precedence(expression: &ast::Expression) -> u8 {
    match expression {
        E::Binary { operator, .. } => operator.precedence(),
        E::Unary { .. } => 8,
        E::Call { .. } => 9,
        _ => 10,
    }
}

impl ast::UnaryOperator {
    const fn text(self) -> &'static str {
        match self {
            Self::Negative => "-",
            Self::Not => "!",
        }
    }

    fn fuses_with(self, operand: &ast::Expression) -> bool {
        matches!((self, operand), (Self::Negative, E::Number(value)) if value.starts_with('-'))
    }
}

impl ast::BinaryOperator {
    const fn text(self) -> &'static str {
        match self {
            B::Or => "||",
            B::And => "&&",
            B::BitOr => "|",
            B::StrictEqual => "===",
            B::StrictNotEqual => "!==",
            B::Less => "<",
            B::LessEqual => "<=",
            B::Greater => ">",
            B::GreaterEqual => ">=",
            B::Add => "+",
            B::Subtract => "-",
            B::Multiply => "*",
            B::Divide => "/",
            B::Remainder => "%",
        }
    }

    const fn precedence(self) -> u8 {
        match self {
            B::Or => 1,
            B::And => 2,
            B::BitOr => 3,
            B::StrictEqual | B::StrictNotEqual => 4,
            B::Less | B::LessEqual | B::Greater | B::GreaterEqual => 5,
            B::Add | B::Subtract => 6,
            B::Multiply | B::Divide | B::Remainder => 7,
        }
    }
}

fn print_string(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{2028}' => output.push_str("\\u2028"),
            '\u{2029}' => output.push_str("\\u2029"),
            character if character <= '\u{1f}' => {
                write!(output, "\\u{:04x}", u32::from(character))
                    .expect("writing to String cannot fail");
            }
            character => output.push(character),
        }
    }
    output.push('"');
}
