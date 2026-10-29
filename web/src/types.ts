export type SourceSpan = {
  start: number;
  end: number;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
};

export type Diagnostic = {
  code: string;
  severity: "error" | "warning";
  message: string;
  span: SourceSpan;
  hint?: string | null;
};

export type SourceMapping = {
  generatedStart: number;
  generatedEnd: number;
  source: SourceSpan;
};

export type CompileResponse = {
  ok: boolean;
  brainfuck: string;
  sourceMap: SourceMapping[];
  diagnostics: Diagnostic[];
};

export type ExecutionStatus = "ready" | "running" | "halted" | "failed";

export type RuntimeError = {
  code: string;
  message: string;
  span?: SourceSpan | null;
};

export type ExecutionSnapshot = {
  instructionPointer: number;
  dataPointer: number;
  currentCell: number;
  tapeWindowStart: number;
  tapeWindow: number[];
  tapeWindowPointerOffset: number;
  outputLength: number;
  inputCursor: number;
  totalStepsExecuted: number;
  status: ExecutionStatus;
  currentSpan?: SourceSpan | null;
  lastError?: RuntimeError | null;
  canStepBackward: boolean;
  historyDepth: number;
};

export type WorkerRequest =
  | { type: "compile"; source: string; input: string }
  | { type: "stepForward" }
  | { type: "stepBackward" }
  | { type: "runBatch"; maxSteps?: number }
  | { type: "reset" };

export type WorkerMessage =
  | { type: "compiled"; result: CompileResponse }
  | { type: "state"; state: ExecutionSnapshot }
  | { type: "outputReset" }
  | { type: "outputChunk"; bytes: Uint8Array }
  | { type: "error"; message: string };
