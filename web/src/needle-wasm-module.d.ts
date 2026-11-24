declare module "*.js" {
  export class WasmSession {
    free(): void;
    constructor(source: string, input: Uint8Array, maxInstructions: number);
    setMaxInstructions(maxInstructions: number): void;
    runSteps(maxSteps: number): unknown;
    inspect(): unknown;
    outputBytesFrom(start: number): Uint8Array;
    reset(): unknown;
  }
  export function compileSource(source: string): unknown;
  export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<WebAssembly.Exports>;
}
