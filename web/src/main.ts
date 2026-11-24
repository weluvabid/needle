import "./style.css";
import type {
  CompileResponse,
  ExecutionSnapshot,
  WorkerMessage,
  WorkerRequest,
} from "./types";

type Example = {
  id: string;
  name: string;
  description: string;
  profile: string;
  category: "useful" | "stress";
  source: string;
  input?: string;
};

const examples: Example[] = [
  {
    id: "hello",
    category: "useful",
    name: "Hello, Needle!",
    description: "A compact run through cell mutation and byte output.",
    profile: "Basic execution",
    source: `# Hello, Needle!\npoke 72\nspill\npoke 29\nspill\npoke 7\nspill\nspill\npoke 3\nspill\nnick 67\nspill\nnick 12\nspill\npoke 46\nspill\npoke 23\nspill\nspill\nnick\nspill\npoke 8\nspill\nnick 7\nspill\nnick 68\nspill\nnick 23\nspill`,
  },
  {
    id: "transfer",
    category: "useful",
    name: "Loop transfer",
    description: "Moves a value between adjacent cells with a loop.",
    profile: "Control flow · memory mutation",
    source: `# Transfer five units from cell 0 to cell 1.\npoke 5\nloop\n  nick\n  next\n  poke\n  back\nstitch\nnext\npoke 48\nspill`,
  },
  {
    id: "tape",
    category: "useful",
    name: "Tape pattern",
    description: "Builds a pattern across nine consecutive tape cells.",
    profile: "Pointer traversal · memory",
    source: `poke 17\nnext\npoke 34\nnext\npoke 51\nnext\npoke 68\nnext\npoke 85\nnext\npoke 102\nnext\npoke 119\nnext\npoke 136\nnext\npoke 153\nback 4`,
  },
  {
    id: "echo",
    category: "useful",
    name: "Input echo",
    description: "Reads eight preset bytes and writes them back unchanged.",
    profile: "Input · output",
    input: "Needle42",
    source: `snag\nspill\nsnag\nspill\nsnag\nspill\nsnag\nspill\nsnag\nspill\nsnag\nspill\nsnag\nspill\nsnag\nspill`,
  },
  {
    id: "fibonacci",
    category: "useful",
    name: "Fibonacci byte sequence",
    description: "Emits the first 16 Fibonacci values modulo 256 as a byte stream.",
    profile: "Fibonacci · 16 values",
    source: `# First 16 Fibonacci values modulo 256, emitted as bytes.
spill
poke 1
spill
spill
poke 1
spill
poke 1
spill
poke 2
spill
poke 3
spill
poke 5
spill
poke 8
spill
poke 13
spill
poke 21
spill
poke 34
spill
poke 55
spill
poke 89
spill
nick 112
spill
nick 23
spill`,
  },
  {
    id: "gcd",
    category: "useful",
    name: "Euclidean GCD",
    description: "Runs the subtraction steps for gcd(252, 105) and emits the result, 21.",
    profile: "GCD · 252 and 105",
    source: `# Euclidean algorithm for gcd(252, 105).
# 252 - 105 - 105 = 42
# 105 - 42 - 42 = 21
# 42 - 21 - 21 = 0
poke 252
nick 105
nick 105
poke 63
nick 42
nick 42
poke 29
spill
nick 1
spill`,
  },
  {
    id: "sieve",
    category: "useful",
    name: "Sieve of Eratosthenes",
    description: "Initializes a 32-cell candidate table, marks composites, then emits the surviving primes.",
    profile: "Prime sieve · 2–31",
    source: `# Sieve of Eratosthenes: primes below 32.
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
poke 1
next
back 32
nick
next
nick
back
next 4
nick
back 4
next 6
nick
back 6
next 8
nick
back 8
next 9
nick
back 9
next 10
nick
back 10
next 12
nick
back 12
next 14
nick
back 14
next 15
nick
back 15
next 16
nick
back 16
next 18
nick
back 18
next 20
nick
back 20
next 21
nick
back 21
next 22
nick
back 22
next 24
nick
back 24
next 25
nick
back 25
next 26
nick
back 26
next 27
nick
back 27
next 28
nick
back 28
next 30
nick
back 30
next 2
loop
next 30
poke 50
spill
nick 50
poke 32
spill
nick 32
back 30
nick
stitch
next
loop
next 29
poke 51
spill
nick 51
poke 32
spill
nick 32
back 29
nick
stitch
next
loop
next 28
poke 52
spill
nick 52
poke 32
spill
nick 32
back 28
nick
stitch
next
loop
next 27
poke 53
spill
nick 53
poke 32
spill
nick 32
back 27
nick
stitch
next
loop
next 26
poke 54
spill
nick 54
poke 32
spill
nick 32
back 26
nick
stitch
next
loop
next 25
poke 55
spill
nick 55
poke 32
spill
nick 32
back 25
nick
stitch
next
loop
next 24
poke 56
spill
nick 56
poke 32
spill
nick 32
back 24
nick
stitch
next
loop
next 23
poke 57
spill
nick 57
poke 32
spill
nick 32
back 23
nick
stitch
next
loop
next 22
poke 49
spill
nick 49
poke 48
spill
nick 48
poke 32
spill
nick 32
back 22
nick
stitch
next
loop
next 21
poke 49
spill
nick 49
poke 49
spill
nick 49
poke 32
spill
nick 32
back 21
nick
stitch
next
loop
next 20
poke 49
spill
nick 49
poke 50
spill
nick 50
poke 32
spill
nick 32
back 20
nick
stitch
next
loop
next 19
poke 49
spill
nick 49
poke 51
spill
nick 51
poke 32
spill
nick 32
back 19
nick
stitch
next
loop
next 18
poke 49
spill
nick 49
poke 52
spill
nick 52
poke 32
spill
nick 32
back 18
nick
stitch
next
loop
next 17
poke 49
spill
nick 49
poke 53
spill
nick 53
poke 32
spill
nick 32
back 17
nick
stitch
next
loop
next 16
poke 49
spill
nick 49
poke 54
spill
nick 54
poke 32
spill
nick 32
back 16
nick
stitch
next
loop
next 15
poke 49
spill
nick 49
poke 55
spill
nick 55
poke 32
spill
nick 32
back 15
nick
stitch
next
loop
next 14
poke 49
spill
nick 49
poke 56
spill
nick 56
poke 32
spill
nick 32
back 14
nick
stitch
next
loop
next 13
poke 49
spill
nick 49
poke 57
spill
nick 57
poke 32
spill
nick 32
back 13
nick
stitch
next
loop
next 12
poke 50
spill
nick 50
poke 48
spill
nick 48
poke 32
spill
nick 32
back 12
nick
stitch
next
loop
next 11
poke 50
spill
nick 50
poke 49
spill
nick 49
poke 32
spill
nick 32
back 11
nick
stitch
next
loop
next 10
poke 50
spill
nick 50
poke 50
spill
nick 50
poke 32
spill
nick 32
back 10
nick
stitch
next
loop
next 9
poke 50
spill
nick 50
poke 51
spill
nick 51
poke 32
spill
nick 32
back 9
nick
stitch
next
loop
next 8
poke 50
spill
nick 50
poke 52
spill
nick 52
poke 32
spill
nick 32
back 8
nick
stitch
next
loop
next 7
poke 50
spill
nick 50
poke 53
spill
nick 53
poke 32
spill
nick 32
back 7
nick
stitch
next
loop
next 6
poke 50
spill
nick 50
poke 54
spill
nick 54
poke 32
spill
nick 32
back 6
nick
stitch
next
loop
next 5
poke 50
spill
nick 50
poke 55
spill
nick 55
poke 32
spill
nick 32
back 5
nick
stitch
next
loop
next 4
poke 50
spill
nick 50
poke 56
spill
nick 56
poke 32
spill
nick 32
back 4
nick
stitch
next
loop
next 3
poke 50
spill
nick 50
poke 57
spill
nick 57
poke 32
spill
nick 32
back 3
nick
stitch
next
loop
next 2
poke 51
spill
nick 51
poke 48
spill
nick 48
poke 32
spill
nick 32
back 2
nick
stitch
next
loop
next 1
poke 51
spill
nick 51
poke 49
spill
nick 49
poke 32
spill
nick 32
back 1
nick
stitch
next
poke 10
spill`,
  },
  {
    id: "output-pressure",
    category: "stress",
    name: "Output pressure",
    description: "Writes 65,025 bytes to exercise output buffering and transfer.",
    profile: "65,025 output bytes",
    source: `# 255 outer iterations × 255 output bytes.\npoke 255\nloop\n  next\n  poke 255\n  loop\n    spill\n    nick\n  stitch\n  back\n  nick\nstitch`,
  },
  {
    id: "nested-stress",
    category: "stress",
    name: "Nested-loop stress",
    description: "Three nested 255-count loops. The run exercises three nested loops with more than 16.5 million inner iterations.",
    profile: "16,581,375 theoretical inner iterations",
    source: `# Three nested countdown loops.\npoke 255\nloop\n  next\n  poke 255\n  loop\n    next\n    poke 255\n    loop\n      nick\n    stitch\n    back\n    nick\n    next\n    back\n  stitch\n  back\n  nick\nstitch`,
  },
  {
    id: "compiler-stress",
    category: "stress",
    name: "Compiler expansion",
    description: "Expands a single counted operation to the compiler instruction limit.",
    profile: "1,000,000 compiled instructions",
    source: `# Maximum supported compile-time expansion.\npoke 1000000`,
  },
  {
    id: "boundary",
    category: "stress",
    name: "Tape boundary",
    description: "Attempts to move left of cell zero and demonstrates the runtime boundary error.",
    profile: "Runtime boundary handling",
    source: `back`,
  },
];

const byId = <T extends HTMLElement>(id: string): T => {
  const element = document.getElementById(id);
  if (!element) throw new Error(`Missing required element #${id}`);
  return element as T;
};

const exampleSelect = byId<HTMLSelectElement>("example-select");
const paceSelect = byId<HTMLSelectElement>("pace-select");
const sourceView = byId<HTMLDivElement>("source-view");
const runButton = byId<HTMLButtonElement>("run-button");
const statusBadge = byId<HTMLSpanElement>("run-status");
const description = byId<HTMLParagraphElement>("program-description");
const profile = byId<HTMLSpanElement>("program-profile");
const budgetValue = byId<HTMLSpanElement>("budget-value");
const compiledCount = byId<HTMLElement>("compiled-count");
const stepsCount = byId<HTMLElement>("steps-count");
const outputCount = byId<HTMLElement>("output-count");
const executionTime = byId<HTMLElement>("execution-time");
const stateValue = byId<HTMLElement>("state-value");
const pointerValue = byId<HTMLElement>("pointer-value");
const tapeView = byId<HTMLDivElement>("tape-view");
const outputView = byId<HTMLPreElement>("output-view");
const outputTabs = byId<HTMLDivElement>("output-tabs");
const errorAlert = byId<HTMLDivElement>("runtime-error");

const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
let compiled: CompileResponse | undefined;
let outputBytes = new Uint8Array();
let activeOutputFormat: "ascii" | "hex" | "bytes" = "bytes";
let running = false;
let runGeneration = 0;
let selectedExample = examples[0];
let renderedSource = "";
let sourceRows: HTMLDivElement[] = [];
let activeSourceLine = 0;
let batchStartedAt = 0;
let executionDurationMs = 0;

const paceSettings = {
  "very-careful": { maxSteps: 1, delayMs: 250, maxInstructions: 100_000 },
  careful: { maxSteps: 10, delayMs: 80, maxInstructions: 1_000_000 },
  steady: { maxSteps: 500, delayMs: 24, maxInstructions: 10_000_000 },
  fast: { maxSteps: 5_000, delayMs: 8, maxInstructions: 60_000_000 },
  max: { maxSteps: 100_000, delayMs: 0, maxInstructions: 100_000_000 },
} as const;

for (const [category, label] of [
  ["useful", "Useful programs"],
  ["stress", "Stress testing"],
] as const) {
  const group = document.createElement("optgroup");
  group.label = label;
  for (const example of examples.filter((item) => item.category === category)) {
    const option = document.createElement("option");
    option.value = example.id;
    option.textContent = example.name;
    group.append(option);
  }
  exampleSelect.append(group);
}
exampleSelect.value = selectedExample.id;
loadExample(selectedExample);

exampleSelect.addEventListener("change", () => {
  const example = examples.find((item) => item.id === exampleSelect.value);
  if (example) {
    selectedExample = example;
    loadExample(example);
  }
});

paceSelect.addEventListener("change", () => {
  const pace = paceSettings[paceSelect.value as keyof typeof paceSettings] ?? paceSettings["very-careful"];
  budgetValue.textContent = `${formatBudget(pace.maxInstructions)} instruction budget`;
  if (running) {
    send({ type: "setBudget", maxInstructions: pace.maxInstructions });
  } else {
    loadExample(selectedExample);
  }
});

runButton.addEventListener("click", () => startRun());
outputTabs.addEventListener("click", (event) => {
  const target = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-output-format]");
  if (!target) return;
  activeOutputFormat = target.dataset.outputFormat as typeof activeOutputFormat;
  for (const tab of outputTabs.querySelectorAll<HTMLButtonElement>("[data-output-format]")) {
    tab.classList.toggle("tab-active", tab === target);
    tab.setAttribute("aria-selected", String(tab === target));
  }
  renderOutput();
});

worker.addEventListener("message", (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  switch (message.type) {
    case "compiled":
      compiled = message.result;
      if (!message.result.ok) {
        running = false;
        setStatus("Compile failed", "error");
        showError(message.result.diagnostics.map((item) => item.message).join(" "));
        runButton.disabled = true;
        return;
      }
      compiledCount.textContent = message.result.brainfuck.length.toLocaleString();
      runButton.disabled = false;
      setStatus("Compiled", "success");
      break;
    case "state":
      if (batchStartedAt !== 0) {
        executionDurationMs += performance.now() - batchStartedAt;
        batchStartedAt = 0;
      }
      renderState(message.state);
      if (running && message.state.status === "running") {
        scheduleBatch(runGeneration);
      } else if (message.state.status === "halted") {
        running = false;
        finishRunTiming();
        setStatus("Complete", "success");
        runButton.disabled = false;
      } else if (message.state.status === "failed") {
        running = false;
        finishRunTiming();
        setStatus("Stopped", "error");
        showError(message.state.lastError?.message ?? "Execution stopped.");
        runButton.disabled = false;
      }
      break;
    case "outputReset":
      outputBytes = new Uint8Array();
      renderOutput();
      break;
    case "outputChunk":
      appendOutput(message.bytes);
      break;
    case "error":
      running = false;
      finishRunTiming();
      runButton.disabled = false;
      setStatus("Runtime error", "error");
      showError(message.message);
      break;
  }
});

function loadExample(example: Example): void {
  stopRun();
  selectedExample = example;
  description.textContent = example.description;
  profile.textContent = example.profile;
  renderSource(example.source);
  compiled = undefined;
  outputBytes = new Uint8Array();
  compiledCount.textContent = "—";
  stepsCount.textContent = "0";
  outputCount.textContent = "0 bytes";
  stateValue.textContent = "Ready";
  executionTime.textContent = "—";
  pointerValue.textContent = "0";
  tapeView.replaceChildren();
  outputView.textContent = "Run the selected program to see its output.";
  hideError();
  runButton.disabled = true;
  setStatus("Preparing", "neutral");
  const pace = paceSettings[paceSelect.value as keyof typeof paceSettings] ?? paceSettings["very-careful"];
  budgetValue.textContent = `${formatBudget(pace.maxInstructions)} instruction budget`;
  send({
    type: "compile",
    source: example.source,
    input: example.input ?? "",
    maxInstructions: pace.maxInstructions,
  });
}

function startRun(): void {
  if (!compiled?.ok || running) return;
  stopRun();
  hideError();
  outputBytes = new Uint8Array();
  renderOutput();
  runButton.disabled = true;
  setStatus("Running", "neutral");
  executionTime.textContent = "Running…";
  batchStartedAt = 0;
  executionDurationMs = 0;
  running = true;
  const generation = runGeneration;
  send({ type: "reset" });
  window.setTimeout(() => {
    if (running && generation === runGeneration) {
      runNextBatch();
    }
  }, 0);
}

function scheduleBatch(generation: number): void {
  const pace = paceSettings[paceSelect.value as keyof typeof paceSettings] ?? paceSettings["very-careful"];
  window.setTimeout(() => {
    if (running && generation === runGeneration) {
      runNextBatch();
    }
  }, pace.delayMs);
}

function runNextBatch(): void {
  const pace = paceSettings[paceSelect.value as keyof typeof paceSettings] ?? paceSettings["very-careful"];
  batchStartedAt = performance.now();
  send({ type: "runBatch", maxSteps: pace.maxSteps });
}

function formatBudget(instructions: number): string {
  return instructions < 1_000_000
    ? `${(instructions / 1_000).toLocaleString()}K`
    : `${(instructions / 1_000_000).toLocaleString()}M`;
}

function finishRunTiming(): void {
  if (batchStartedAt !== 0) {
    executionDurationMs += performance.now() - batchStartedAt;
    batchStartedAt = 0;
  }
  executionTime.textContent = `${executionDurationMs.toFixed(1)} ms`;
}

function stopRun(): void {
  running = false;
  runGeneration += 1;
}

function send(request: WorkerRequest): void {
  worker.postMessage(request);
}

function appendOutput(chunk: Uint8Array): void {
  const next = new Uint8Array(outputBytes.length + chunk.length);
  next.set(outputBytes);
  next.set(chunk, outputBytes.length);
  outputBytes = next;
  outputCount.textContent = `${outputBytes.length.toLocaleString()} bytes`;
  renderOutput();
}

function renderOutput(): void {
  if (outputBytes.length === 0) {
    outputView.textContent = "Nothing spilled yet.";
    return;
  }
  if (activeOutputFormat === "ascii") {
    outputView.textContent = Array.from(outputBytes, (byte) => {
      if (byte === 10) return "\\n";
      if (byte === 13) return "\\r";
      if (byte === 9) return "\\t";
      return byte >= 32 && byte <= 126 ? String.fromCharCode(byte) : "·";
    }).join("");
  } else if (activeOutputFormat === "hex") {
    outputView.textContent = Array.from(outputBytes, (byte) => byte.toString(16).padStart(2, "0")).join(" ");
  } else {
    outputView.textContent = Array.from(outputBytes, (byte) => String(byte)).join(" ");
  }
}

function renderSource(source: string): void {
  if (renderedSource === source) {
    renderSourceLine(0);
    return;
  }

  renderedSource = source;
  activeSourceLine = 0;
  sourceRows = [];
  sourceView.replaceChildren();

  source.split("\n").forEach((line, index) => {
    const row = document.createElement("div");
    row.className = "flex min-w-max gap-4 rounded px-2 py-1";
    row.dataset.line = String(index + 1);

    const lineNumber = document.createElement("span");
    lineNumber.className = "w-6 shrink-0 select-none text-right text-base-content/40";
    lineNumber.textContent = String(index + 1);

    const code = document.createElement("span");
    code.className = "whitespace-pre text-base-content";
    code.textContent = line || " ";

    row.append(lineNumber, code);
    sourceView.append(row);
    sourceRows.push(row);
  });

  renderSourceLine(0);
}

function renderSourceLine(line: number): void {
  const previousLine = activeSourceLine;
  if (line > 0) activeSourceLine = line;
  if (previousLine === activeSourceLine) return;

  for (const row of sourceRows) {
    const active = Number(row.dataset.line) === activeSourceLine;
    row.classList.toggle("bg-primary/10", active);
    row.classList.toggle("border-l-2", active);
    row.classList.toggle("border-primary", active);
    row.setAttribute("aria-current", active ? "step" : "false");
    if (active) row.scrollIntoView({ block: "nearest" });
  }
}

function renderState(state: ExecutionSnapshot): void {
  if (state.currentSpan) {
    renderSourceLine(state.currentSpan.line);
  }
  stateValue.textContent = state.status[0].toUpperCase() + state.status.slice(1);
  pointerValue.textContent = state.dataPointer.toLocaleString();
  stepsCount.textContent = state.totalStepsExecuted.toLocaleString();
  outputCount.textContent = `${state.outputLength.toLocaleString()} bytes`;
  tapeView.replaceChildren();
  state.tapeWindow.forEach((value, index) => {
    const cellIndex = state.tapeWindowStart + index;
    const cell = document.createElement("div");
    cell.className = `card bg-base-200 border border-base-300 min-w-0 p-3 text-center${index === state.tapeWindowPointerOffset ? " border-primary bg-base-300" : ""}`;
    const valueNode = document.createElement("strong");
    valueNode.className = "font-mono tabular-nums";
    valueNode.textContent = String(value);
    const indexNode = document.createElement("span");
    indexNode.className = "text-xs text-base-content/50 font-mono";
    indexNode.textContent = String(cellIndex);
    cell.append(valueNode, indexNode);
    tapeView.append(cell);
  });
}

function setStatus(message: string, variant: "success" | "error" | "neutral"): void {
  statusBadge.textContent = message;
  statusBadge.className = `badge badge-soft shrink-0 ${variant === "success" ? "badge-success" : variant === "error" ? "badge-error" : "badge-ghost"}`;
}

function showError(message: string): void {
  errorAlert.textContent = message;
  errorAlert.classList.remove("hidden");
}

function hideError(): void {
  errorAlert.textContent = "";
  errorAlert.classList.add("hidden");
}
