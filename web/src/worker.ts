/// <reference lib="webworker" />
import init, { compileSource, WasmSession } from "./wasm/needle_wasm.js";
import type { CompileResponse, ExecutionSnapshot, WorkerRequest, WorkerMessage } from "./types";

const scope = self as DedicatedWorkerGlobalScope;
let wasmReady: Promise<unknown> | undefined;
let session: WasmSession | undefined;
let lastOutputLength = 0;
let requestQueue = Promise.resolve();

scope.addEventListener("message", (event: MessageEvent<WorkerRequest>) => {
  requestQueue = requestQueue
    .then(() => handleRequest(event.data))
    .catch((error: unknown) => {
      post({ type: "error", message: error instanceof Error ? error.message : String(error) });
    });
});

async function handleRequest(request: WorkerRequest): Promise<void> {
  wasmReady ??= init();
  await wasmReady;

  switch (request.type) {
    case "compile": {
      session?.free();
      session = undefined;
      lastOutputLength = 0;
      post({ type: "outputReset" });
      const result = compileSource(request.source) as CompileResponse;
      post({ type: "compiled", result });
      if (!result.ok) {
        return;
      }
      const input = new TextEncoder().encode(request.input);
      if (input.length > 1024 * 1024) {
        post({ type: "error", message: "Input is limited to 1 MiB after UTF-8 encoding." });
        return;
      }
      session = new WasmSession(request.source, input);
      publishState(session.inspect() as ExecutionSnapshot);
      return;
    }
    case "stepForward": {
      if (!session) return;
      publishState(session.stepForward() as ExecutionSnapshot);
      return;
    }
    case "stepBackward": {
      if (!session) return;
      const result = session.stepBackward() as { changed: boolean; state: ExecutionSnapshot };
      publishState(result.state);
      return;
    }
    case "runBatch": {
      if (!session) return;
      const result = session.runSteps(request.maxSteps ?? 500) as { executedThisBatch: number; state: ExecutionSnapshot };
      publishState(result.state);
      return;
    }
    case "reset": {
      if (!session) return;
      lastOutputLength = 0;
      post({ type: "outputReset" });
      publishState(session.reset() as ExecutionSnapshot);
      return;
    }
  }
}

function publishState(state: ExecutionSnapshot): void {
  if (state.outputLength < lastOutputLength) {
    lastOutputLength = 0;
    post({ type: "outputReset" });
  }
  if (session && state.outputLength > lastOutputLength) {
    const chunk = session.outputBytesFrom(lastOutputLength);
    lastOutputLength = state.outputLength;
    post({ type: "outputChunk", bytes: chunk });
  }
  post({ type: "state", state });
}

function post(message: WorkerMessage): void {
  if (message.type === "outputChunk") {
    scope.postMessage(message, [message.bytes.buffer]);
  } else {
    scope.postMessage(message);
  }
}
