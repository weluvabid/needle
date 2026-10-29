import "./style.css";
import type {
  CompileResponse,
  Diagnostic,
  ExecutionSnapshot,
  WorkerMessage,
  WorkerRequest,
} from "./types";

const byId = <T extends HTMLElement>(id: string): T => {
  const element = document.getElementById(id);
  if (!element) throw new Error(`Missing required element #${id}`);
  return element as T;
};

const sourceEditor = byId<HTMLTextAreaElement>("source-editor");
const sourceHighlight = byId<HTMLPreElement>("source-highlight");
const lineNumbers = byId<HTMLDivElement>("line-numbers");
const brainfuckOutput = byId<HTMLPreElement>("brainfuck-output");
const diagnostics = byId<HTMLDivElement>("diagnostics");
const compileStatus = byId<HTMLSpanElement>("compile-status");
const sourceCount = byId<HTMLSpanElement>("source-count");
const bfCount = byId<HTMLSpanElement>("bf-count");
const tapeView = byId<HTMLDivElement>("tape-view");
const pointerLabel = byId<HTMLSpanElement>("pointer-label");
const programOutput = byId<HTMLPreElement>("program-output");
const outputCount = byId<HTMLSpanElement>("output-count");
const programInput = byId<HTMLTextAreaElement>("program-input");
const stateValue = byId<HTMLElement>("state-value");
const instructionValue = byId<HTMLElement>("instruction-value");
const cellValue = byId<HTMLElement>("cell-value");
const stepsValue = byId<HTMLElement>("steps-value");
const budgetBar = byId<HTMLSpanElement>("budget-bar");
const toast = byId<HTMLDivElement>("toast");
const threadButton = byId<HTMLButtonElement>("thread-button");
const runButton = byId<HTMLButtonElement>("run-button");
const pauseButton = byId<HTMLButtonElement>("pause-button");
const stepButton = byId<HTMLButtonElement>("step-button");
const backButton = byId<HTMLButtonElement>("back-button");
const resetButton = byId<HTMLButtonElement>("reset-button");
const clearButton = byId<HTMLButtonElement>("clear-button");
const copyButton = byId<HTMLButtonElement>("copy-bf-button");
const speedControl = byId<HTMLSelectElement>("speed-control");

const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
let decoder = new TextDecoder("utf-8", { fatal: false });
let compiled: CompileResponse | undefined;
let currentState: ExecutionSnapshot | undefined;
let isRunning = false;
let executionReady = false;
let compileTimer: ReturnType<typeof setTimeout> | undefined;
let toastTimer: ReturnType<typeof setTimeout> | undefined;
let lastCompileKey = "";
let dirtyInput = false;
let renderedBrainfuckCode: string | undefined;
let pendingAction: "run" | "stepForward" | "stepBackward" | "reset" | null = null;

const STARTER = `# Say hello, one cell at a time.\npoke 72\nspill\npoke 33\nspill\nnick 72\nspill\n`;
sourceEditor.value = STARTER;
programInput.value = "";
renderEditor();
compileNow();

worker.addEventListener("message", (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  switch (message.type) {
    case "compiled":
      compiled = message.result;
      isRunning = false;
      executionReady = false;
      renderCompilation(message.result);
      setCompileStatus(message.result.ok ? "Threaded cleanly" : "Loose threads found", message.result.ok ? "good" : "error");
      if (message.result.ok) {
        currentState = undefined;
        renderBrainfuck();
      }
      if (!message.result.ok) pendingAction = null;
      updateControls();
      break;
    case "state":
      currentState = message.state;
      executionReady = true;
      renderState(message.state);
      if (isRunning && message.state.status === "running") {
        window.setTimeout(() => {
          if (isRunning) send({ type: "runBatch", maxSteps: pace().batchSize });
        }, pace().delayMs);
      } else if (message.state.status === "halted" || message.state.status === "failed") {
        isRunning = false;
        programOutput.textContent += decoder.decode();
      }
      outputCount.textContent = `${message.state.outputLength} bytes`;
      if (pendingAction && message.state.status !== "failed") {
        const action = pendingAction;
        pendingAction = null;
        performAction(action);
      }
      updateControls();
      break;
    case "outputReset":
      decoder = new TextDecoder("utf-8", { fatal: false });
      programOutput.textContent = "Nothing spilled yet.";
      outputCount.textContent = "0 bytes";
      break;
    case "outputChunk": {
      if (programOutput.textContent === "Nothing spilled yet.") programOutput.textContent = "";
      programOutput.textContent += decoder.decode(message.bytes, { stream: true });
      outputCount.textContent = `${currentState?.outputLength ?? message.bytes.length} bytes`;
      break;
    }
    case "error":
      executionReady = false;
      pendingAction = null;
      setCompileStatus("Worker snagged", "error");
      showToast(message.message);
      isRunning = false;
      updateControls();
      break;
  }
});

sourceEditor.addEventListener("input", () => {
  isRunning = false;
  pendingAction = null;
  currentState = undefined;
  executionReady = false;
  renderEditor();
  updateControls();
  scheduleCompile();
});
sourceEditor.addEventListener("scroll", () => syncEditorScroll());
sourceEditor.addEventListener("keydown", (event: KeyboardEvent) => {
  if (event.key === "Tab") {
    event.preventDefault();
    insertText("  ");
  }
  if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
    event.preventDefault();
    queueAction("run");
  }
});
programInput.addEventListener("input", () => {
  isRunning = false;
  pendingAction = null;
  currentState = undefined;
  executionReady = false;
  dirtyInput = true;
  updateControls();
  scheduleCompile();
});

threadButton.addEventListener("click", () => { pendingAction = null; compileNow(); });
runButton.addEventListener("click", () => queueAction("run"));
pauseButton.addEventListener("click", () => {
  isRunning = false;
  updateControls();
  setCompileStatus("Paused on the thread", "neutral");
});
stepButton.addEventListener("click", () => queueAction("stepForward"));
backButton.addEventListener("click", () => queueAction("stepBackward"));
resetButton.addEventListener("click", () => queueAction("reset"));
clearButton.addEventListener("click", () => {
  sourceEditor.value = "";
  sourceEditor.focus();
  renderEditor();
  compileNow();
});
copyButton.addEventListener("click", async () => {
  if (!compiled?.ok) return;
  try {
    await navigator.clipboard.writeText(compiled.brainfuck);
    showToast("Brainfuck copied to the clipboard.");
  } catch {
    showToast("Clipboard access was blocked by the browser.");
  }
});

function send(request: WorkerRequest): void {
  worker.postMessage(request);
}

function pace(): { delayMs: number; batchSize: number } {
  switch (speedControl.value) {
    case "slow": return { delayMs: 160, batchSize: 1 };
    case "fast": return { delayMs: 0, batchSize: 5_000 };
    default: return { delayMs: 16, batchSize: 500 };
  }
}

function scheduleCompile(): void {
  if (compileTimer) clearTimeout(compileTimer);
  compileTimer = setTimeout(compileNow, 180);
}

function compileNow(): void {
  if (compileTimer) clearTimeout(compileTimer);
  isRunning = false;
  lastCompileKey = `${sourceEditor.value}\u0000${programInput.value}`;
  dirtyInput = false;
  currentState = undefined;
  executionReady = false;
  send({ type: "compile", source: sourceEditor.value, input: programInput.value });
  setCompileStatus("Threading…", "neutral");
  updateControls();
}

function queueAction(action: "run" | "stepForward" | "stepBackward" | "reset"): void {
  const key = `${sourceEditor.value}\u0000${programInput.value}`;
  if (key !== lastCompileKey || dirtyInput || !compiled) {
    pendingAction = action;
    compileNow();
    return;
  }
  if (!compiled.ok) return;
  if (!executionReady) {
    pendingAction = action;
    return;
  }
  performAction(action);
}

function performAction(action: "run" | "stepForward" | "stepBackward" | "reset"): void {
  if (action === "run") {
    isRunning = true;
    updateControls();
    send({ type: "runBatch", maxSteps: pace().batchSize });
    return;
  }
  if (action === "stepBackward" || action === "reset") isRunning = false;
  if (action === "stepForward") send({ type: "stepForward" });
  else if (action === "stepBackward") send({ type: "stepBackward" });
  else send({ type: "reset" });
  updateControls();
}

function renderCompilation(result: CompileResponse): void {
  if (!result.ok) {
    renderDiagnostics(result.diagnostics);
    brainfuckOutput.innerHTML = '<span class="placeholder">Fix the loose threads to compile this source.</span>';
    bfCount.textContent = "—";
    copyButton.disabled = true;
    renderBrainfuck();
    return;
  }
  renderDiagnostics([]);
  bfCount.textContent = `${result.brainfuck.length.toLocaleString()} instructions`;
  copyButton.disabled = false;
  renderBrainfuck();
}

function renderDiagnostics(items: Diagnostic[]): void {
  diagnostics.replaceChildren();
  if (items.length === 0) {
    diagnostics.className = "diagnostics is-clean";
    const icon = document.createElement("span");
    icon.className = "diagnostics-icon";
    icon.textContent = "✓";
    const text = document.createElement("span");
    text.textContent = "No loose threads.";
    diagnostics.append(icon, text);
    return;
  }
  diagnostics.className = "diagnostics has-errors";
  const icon = document.createElement("span");
  icon.className = "diagnostics-icon";
  icon.textContent = "!";
  diagnostics.append(icon);
  for (const diagnostic of items) {
    const item = document.createElement("button");
    item.className = "diagnostic-item";
    item.type = "button";
    item.textContent = `${diagnostic.span.line}:${diagnostic.span.column}  ${diagnostic.message}${diagnostic.hint ? ` ${diagnostic.hint}` : ""}`;
    item.addEventListener("click", () => focusSpan(diagnostic.span.start, diagnostic.span.end));
    diagnostics.append(item);
  }
}

function renderEditor(): void {
  const source = sourceEditor.value;
  const lines = source.split("\n");
  sourceCount.textContent = `${lines.length} lines`;
  lineNumbers.textContent = lines.map((_, index) => String(index + 1)).join("\n");
  sourceHighlight.innerHTML = highlightSource(source, currentState?.currentSpan?.line);
  syncEditorScroll();
}

function highlightSource(source: string, activeLine?: number | null): string {
  const escaped = source.split("\n").map((line, index) => {
    const commentIndex = line.indexOf("#");
    const codePart = commentIndex >= 0 ? line.slice(0, commentIndex) : line;
    const commentPart = commentIndex >= 0 ? line.slice(commentIndex) : "";
    const highlighted = escapeHtml(codePart).replace(/\b(poke|nick|next|back|loop|stitch|spill|snag)\b/g, '<span class="syntax-command">$1</span>')
      .replace(/\b(\d+)\b/g, '<span class="syntax-number">$1</span>');
    const lineHtml = `${highlighted}${commentPart ? `<span class="syntax-comment">${escapeHtml(commentPart)}</span>` : ""}` || " ";
    return activeLine === index + 1 ? `<span class="active-source-line">${lineHtml}</span>` : lineHtml;
  }).join("\n");
  return `${escaped}\n`;
}

function renderBrainfuck(): void {
  const code = compiled?.ok ? compiled.brainfuck : "";
  const span = currentState?.currentSpan;
  const activeMapping = compiled?.ok && currentState && span
    ? compiled.sourceMap[currentState.instructionPointer]
    : undefined;

  if (!code) {
    brainfuckOutput.textContent = compiled?.ok
      ? ""
      : compiled
        ? "Fix the loose threads to compile this source."
        : "Your compiled instructions will appear here.";
    brainfuckOutput.classList.toggle("is-placeholder", !compiled?.ok);
    renderedBrainfuckCode = undefined;
    clearBrainfuckHighlight();
    return;
  }

  brainfuckOutput.classList.remove("is-placeholder");
  if (renderedBrainfuckCode !== code) {
    brainfuckOutput.textContent = code;
    renderedBrainfuckCode = code;
  }

  if (!activeMapping || activeMapping.generatedEnd <= activeMapping.generatedStart) {
    clearBrainfuckHighlight();
    return;
  }

  const textNode = brainfuckOutput.firstChild;
  const highlightRegistry = (CSS as typeof CSS & {
    highlights?: { set(name: string, highlight: unknown): void; delete(name: string): void };
  }).highlights;
  const Highlight = (window as unknown as {
    Highlight?: new (...ranges: Range[]) => unknown;
  }).Highlight;

  if (textNode?.nodeType === Node.TEXT_NODE && highlightRegistry && Highlight) {
    const range = document.createRange();
    range.setStart(textNode, activeMapping.generatedStart);
    range.setEnd(textNode, activeMapping.generatedEnd);
    highlightRegistry.set("needle-active-bf", new Highlight(range));
    return;
  }

  const before = code.slice(0, activeMapping.generatedStart);
  const active = code.slice(activeMapping.generatedStart, activeMapping.generatedEnd);
  const after = code.slice(activeMapping.generatedEnd);
  brainfuckOutput.innerHTML = `${escapeHtml(before)}<span class="active-bf-range">${escapeHtml(active)}</span>${escapeHtml(after)}`;
  renderedBrainfuckCode = undefined;
}

function clearBrainfuckHighlight(): void {
  const highlightRegistry = (CSS as typeof CSS & {
    highlights?: { delete(name: string): void };
  }).highlights;
  highlightRegistry?.delete("needle-active-bf");
  if (brainfuckOutput.querySelector(".active-bf-range")) {
    const code = compiled?.ok ? compiled.brainfuck : "";
    brainfuckOutput.textContent = code;
    renderedBrainfuckCode = code || undefined;
  }
}

function renderState(state: ExecutionSnapshot): void {
  const previousLine = sourceHighlight.dataset.activeLine;
  const currentLine = state.currentSpan?.line ? String(state.currentSpan.line) : "";
  if (previousLine !== currentLine) {
    sourceHighlight.dataset.activeLine = currentLine;
    sourceHighlight.innerHTML = highlightSource(sourceEditor.value, state.currentSpan?.line);
  }
  renderBrainfuck();
  renderTape(state);
  stateValue.textContent = state.status[0].toUpperCase() + state.status.slice(1);
  stateValue.className = state.status === "failed" ? "state-error" : state.status === "halted" ? "state-halted" : "";
  instructionValue.textContent = state.currentSpan ? `${state.currentSpan.line}:${state.currentSpan.column}` : state.status === "halted" ? "End of thread" : "—";
  cellValue.innerHTML = `${state.currentCell} <small>/ 255</small>`;
  stepsValue.innerHTML = `${state.totalStepsExecuted.toLocaleString()} <small>/ 1,000,000</small>`;
  budgetBar.style.width = `${Math.min(100, state.totalStepsExecuted / 1_000_000 * 100)}%`;
  pointerLabel.textContent = `pointer: ${state.dataPointer}`;
  outputCount.textContent = `${state.outputLength} bytes`;
  if (state.lastError) {
    setCompileStatus(state.lastError.message, "error");
  } else if (state.status === "halted") {
    setCompileStatus("Thread complete", "good");
  } else if (state.status === "running") {
    setCompileStatus(isRunning ? "Following the thread" : "Paused on the thread", "neutral");
  } else {
    setCompileStatus("Ready to thread", "neutral");
  }
  updateControls();
}

function renderTape(state: ExecutionSnapshot): void {
  tapeView.replaceChildren();
  state.tapeWindow.forEach((value, index) => {
    const cellIndex = state.tapeWindowStart + index;
    const cell = document.createElement("div");
    cell.className = `tape-cell${index === state.tapeWindowPointerOffset ? " is-current" : ""}${value === 0 ? " is-zero" : ""}`;
    const valueNode = document.createElement("strong");
    valueNode.textContent = String(value);
    const indexNode = document.createElement("small");
    indexNode.textContent = String(cellIndex);
    cell.append(valueNode, indexNode);
    tapeView.append(cell);
  });
}

function updateControls(): void {
  const valid = Boolean(compiled?.ok) && executionReady;
  const terminal = currentState?.status === "halted" || currentState?.status === "failed";
  runButton.disabled = !valid || isRunning || terminal;
  pauseButton.disabled = !isRunning;
  stepButton.disabled = !valid || isRunning || terminal;
  backButton.disabled = !valid || isRunning || !currentState?.canStepBackward;
  resetButton.disabled = !valid;
  threadButton.disabled = false;
}

function setCompileStatus(message: string, variant: "good" | "error" | "neutral"): void {
  compileStatus.textContent = message;
  compileStatus.className = `status-label status-${variant}`;
}

function insertText(value: string): void {
  const start = sourceEditor.selectionStart;
  const end = sourceEditor.selectionEnd;
  sourceEditor.setRangeText(value, start, end, "end");
  sourceEditor.dispatchEvent(new Event("input", { bubbles: true }));
}

function focusSpan(start: number, end: number): void {
  sourceEditor.focus();
  const source = sourceEditor.value;
  const selectionStart = utf8OffsetToUtf16(source, start);
  const selectionEnd = utf8OffsetToUtf16(source, end);
  sourceEditor.setSelectionRange(selectionStart, selectionEnd);
  const before = source.slice(0, selectionStart);
  const line = before.split("\n").length - 1;
  sourceEditor.scrollTop = Math.max(0, line * 24 - 60);
  syncEditorScroll();
}

function utf8OffsetToUtf16(source: string, byteOffset: number): number {
  let bytes = 0;
  let utf16 = 0;
  for (const character of source) {
    if (bytes >= byteOffset) break;
    const codePoint = character.codePointAt(0) ?? 0;
    bytes += codePoint <= 0x7f ? 1 : codePoint <= 0x7ff ? 2 : codePoint <= 0xffff ? 3 : 4;
    utf16 += codePoint > 0xffff ? 2 : 1;
  }
  return utf16;
}

function syncEditorScroll(): void {
  sourceHighlight.scrollTop = sourceEditor.scrollTop;
  sourceHighlight.scrollLeft = sourceEditor.scrollLeft;
  lineNumbers.scrollTop = sourceEditor.scrollTop;
}

function showToast(message: string): void {
  toast.textContent = message;
  toast.classList.add("is-visible");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove("is-visible"), 2600);
}

function escapeHtml(value: string): string {
  const entities: Record<string, string> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  };
  return value.replace(/[&<>"']/g, (character) => entities[character] ?? character);
}
