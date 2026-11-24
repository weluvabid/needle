//! WebAssembly bindings exposing Needle parsing, compilation, and execution
//! to the browser-based playground.

use js_sys::Uint8Array;
use needle_core::{
    Diagnostic, ExecutionLimits, ExecutionSession, ExecutionSnapshot, MAX_INPUT_BYTES, Program,
    RuntimeError, SourceMapping, compile, parse,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompileResponse {
    ok: bool,
    brainfuck: String,
    source_map: Vec<SourceMapping>,
    diagnostics: Vec<Diagnostic>,
}

/// Compile Needle source, returning diagnostics or generated Brainfuck and source mappings.
#[wasm_bindgen(js_name = compileSource)]
pub fn compile_source(source: &str) -> JsValue {
    let response = match compile(source) {
        Ok(compilation) => CompileResponse {
            ok: true,
            brainfuck: compilation.brainfuck,
            source_map: compilation.source_map,
            diagnostics: Vec::new(),
        },
        Err(diagnostics) => CompileResponse {
            ok: false,
            brainfuck: String::new(),
            source_map: Vec::new(),
            diagnostics,
        },
    };
    serialize(&response)
}

/// An owned interpreter session exposed to the TypeScript worker.
#[wasm_bindgen]
pub struct WasmSession {
    inner: ExecutionSession,
}

#[wasm_bindgen]
impl WasmSession {
    /// Parse source and create a session with the selected instruction budget.
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str, input: &[u8], max_instructions: u32) -> Result<WasmSession, JsValue> {
        if input.len() > MAX_INPUT_BYTES {
            return Err(serialize(&RuntimeError {
                code: needle_core::RuntimeErrorCode::InputLimitExceeded,
                message: format!("Input is limited to {MAX_INPUT_BYTES} bytes."),
                span: None,
            }));
        }
        let program: Program = parse(source).map_err(|diagnostics| serialize(&diagnostics))?;
        let limits = ExecutionLimits {
            max_instructions: u64::from(max_instructions),
            ..ExecutionLimits::default()
        };
        let inner = ExecutionSession::new(program, input.to_vec(), limits)
            .map_err(|error| serialize(&error))?;
        Ok(Self { inner })
    }

    /// Change the active instruction budget without resetting the session.
    #[wasm_bindgen(js_name = setMaxInstructions)]
    pub fn set_max_instructions(&mut self, max_instructions: u32) -> Result<(), JsValue> {
        self.inner
            .set_max_instructions(u64::from(max_instructions))
            .map_err(|error| serialize(&error))
    }

    /// Execute a bounded batch of source operations and return state.
    #[wasm_bindgen(js_name = runSteps)]
    pub fn run_steps(&mut self, max_steps: u32) -> JsValue {
        let executed_this_batch = self.inner.run_steps(max_steps as usize);
        serialize(&RunResponse {
            executed_this_batch,
            state: self.inner.inspect(),
        })
    }

    /// Inspect current interpreter state without copying the accumulated output.
    pub fn inspect(&self) -> JsValue {
        serialize(&self.inner.inspect())
    }

    /// Return only output bytes at or after `start`, avoiding repeated full-buffer copies.
    #[wasm_bindgen(js_name = outputBytesFrom)]
    pub fn output_bytes_from(&self, start: u32) -> Uint8Array {
        Uint8Array::from(self.inner.output_bytes_from(start as usize))
    }

    /// Reset memory, cursors, output, history, and the execution budget.
    pub fn reset(&mut self) -> JsValue {
        self.inner.reset();
        serialize(&self.inner.inspect())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunResponse {
    executed_this_batch: usize,
    state: ExecutionSnapshot,
}

fn serialize<T: Serialize>(value: &T) -> JsValue {
    serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL)
}
