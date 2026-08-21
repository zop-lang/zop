// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Command-line driver for compiling one Zop source file.
//!
//! The driver selects one explicit artifact mode. Compilation failures are
//! rendered once at the process boundary and never retried through another
//! backend.

use std::{env, fs, process::ExitCode};

use zop::{backend, diagnostic::Diagnostics, frontend::analyze, mlir};

/// Explicit artifact selected by the command line.
enum Command {
    /// Deterministic ECMAScript module emission.
    JavaScript {
        /// Zop source path read by the driver.
        source: String,

        /// Destination path for the generated module.
        output: String,
    },

    /// Verified textual Multi-Level Intermediate Representation output.
    Mlir {
        /// Zop source path read by the driver.
        source: String,
    },

    /// Native object emission through Cranelift.
    Object {
        /// Zop source path read by the driver.
        source: String,

        /// Destination path for the native object.
        output: String,
    },
}

fn main() -> ExitCode {
    match parse_command().and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_command() -> Result<Command, String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, source, output] if command == "javascript" => {
            Ok(Command::JavaScript { source: source.clone(), output: output.clone() })
        }
        [command, source] if command == "mlir" => Ok(Command::Mlir { source: source.clone() }),
        [command, source, output] if command == "object" => {
            Ok(Command::Object { source: source.clone(), output: output.clone() })
        }
        _ => Err(
            "usage: zop javascript <source> <output>\n       zop mlir <source>\n       zop object <source> <output>"
                .into(),
        ),
    }
}

fn run(command: Command) -> Result<(), String> {
    let source_path = match &command {
        Command::JavaScript { source, .. }
        | Command::Mlir { source }
        | Command::Object { source, .. } => source,
    };
    let source = fs::read_to_string(source_path)
        .map_err(|error| format!("failed to read {source_path}: {error}"))?;
    let hir = analyze(&source).map_err(format_diagnostics)?;

    match command {
        Command::JavaScript { output, .. } => {
            let javascript = backend::javascript_text(&hir).map_err(format_diagnostics)?;
            fs::write(&output, javascript)
                .map_err(|error| format!("failed to write {output}: {error}"))?;
        }
        Command::Mlir { .. } => {
            println!("{}", mlir::mlir_text(&hir).map_err(format_diagnostics)?);
        }
        Command::Object { output, .. } => {
            let object = backend::compile_object(&hir).map_err(format_diagnostics)?;
            fs::write(&output, object)
                .map_err(|error| format!("failed to write {output}: {error}"))?;
        }
    }
    Ok(())
}

fn format_diagnostics(errors: Diagnostics) -> String {
    errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join("\n")
}
