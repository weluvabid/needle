declare module "*.js" {
  export class WasmSession {
    free(): void;
    constructor(source: string, input: Uint8Array);
    stepForward(): unknown;
    runSteps(maxSteps: number): unknown;
    stepBackward(): unknown;
    inspect(): unknown;
    outputBytesFrom(start: number): Uint8Array;
    reset(): unknown;
  }
  export function compileSource(source: string): unknown;
  export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<WebAssembly.Exports>;
}
