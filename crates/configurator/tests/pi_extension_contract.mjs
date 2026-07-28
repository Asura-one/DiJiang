import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const extensionPath = resolve(process.argv[2] || "");
if (!process.argv[2]) {
  throw new Error("usage: node pi_extension_contract.mjs <extension-path>");
}

function piPackageDir() {
  if (process.env.PI_CODING_AGENT_PACKAGE_DIR) {
    return process.env.PI_CODING_AGENT_PACKAGE_DIR;
  }
  return join(execFileSync("npm", ["root", "-g"], { encoding: "utf8" }).trim(), "@earendil-works", "pi-coding-agent");
}

const packageDir = piPackageDir();
const pi = await import(pathToFileURL(join(packageDir, "dist", "index.js")).href);
const { loadExtensions } = await import(pathToFileURL(join(packageDir, "dist", "core", "extensions", "loader.js")).href);
const tempRoot = mkdtempSync(join(tmpdir(), "dijiang-pi-contract-"));
const binDir = join(tempRoot, "bin");
const fakeDijiang = join(binDir, "dijiang");
const originalPath = process.env.PATH;

function workflowState(gate) {
  return JSON.stringify({
    state: {
      activeTask: { id: "pi-contract", title: "Pi contract", status: "in_progress" },
      routeGate: { capsule: "implement" },
      gitGate: { state: gate, worktreePath: "/tmp/dijiang-task-worktree" },
    },
  });
}

function event(toolName, input) {
  return {
    type: "tool_call",
    toolCallId: `contract-${toolName}`,
    toolName,
    input,
  };
}

try {
  mkdirSync(binDir);
  writeFileSync(fakeDijiang, `#!/bin/sh\nprintf '%s\n' "$DIJIANG_PI_CONTRACT_STATE"\n`);
  chmodSync(fakeDijiang, 0o755);
  process.env.PATH = `${binDir}:${originalPath}`;
  execFileSync("git", ["init", "-q"], { cwd: tempRoot });
  writeFileSync(join(tempRoot, "dirty.txt"), "dirty\n");

  const loaded = await loadExtensions([extensionPath], tempRoot);
  assert.deepEqual(loaded.errors, [], "Pi must load the generated DiJiang extension without runtime errors");

  const runner = new pi.ExtensionRunner(
    loaded.extensions,
    loaded.runtime,
    tempRoot,
    pi.SessionManager.inMemory(tempRoot),
    {},
  );
  runner.bindCore(
    {
      sendMessage: () => {},
      sendUserMessage: () => {},
      appendEntry: () => {},
      setSessionName: () => {},
      getSessionName: () => undefined,
      setLabel: () => {},
      getActiveTools: () => [],
      getAllTools: () => [],
      setActiveTools: () => {},
      refreshTools: () => {},
      getCommands: () => [],
      setModel: async () => {},
      getThinkingLevel: () => undefined,
      setThinkingLevel: () => {},
    },
    {
      getModel: () => undefined,
      isIdle: () => true,
      isProjectTrusted: () => true,
      getSignal: () => undefined,
      abort: () => {},
      hasPendingMessages: () => false,
      shutdown: () => {},
      getContextUsage: () => undefined,
      compact: () => {},
      getSystemPrompt: () => "",
      getSystemPromptOptions: () => ({ cwd: tempRoot }),
    },
  );
  const extensionErrors = [];
  runner.onError((error) => extensionErrors.push(error));

  process.env.DIJIANG_PI_CONTRACT_STATE = workflowState("blocked");

  const blockedWrite = await runner.emitToolCall(event("write", { path: "src/main.rs", content: "x" }));
  assert.deepEqual(extensionErrors, [], `Pi extension handler errors: ${JSON.stringify(extensionErrors)}`);
  assert.equal(blockedWrite?.block, true, "blocked Git Gate must block write tool calls");
  assert.match(blockedWrite?.reason ?? "", /Git Gate/);

  const blockedBash = await runner.emitToolCall(event("bash", { command: "printf changed" }));
  assert.equal(blockedBash?.block, true, "blocked Git Gate must block potentially mutating bash commands");

  const readOnlyBash = event("bash", { command: "git status --short" });
  assert.equal(await runner.emitToolCall(readOnlyBash), undefined, "blocked Git Gate must allow read-only bash inspection");
  assert.match(readOnlyBash.input.command, /^export DIJIANG_CONTEXT_ID=/, "allowed bash calls must receive the Pi session context");
  assert.equal(
    (await runner.emitToolCall(event("bash", { command: "git diff --output=/tmp/changed" })))?.block,
    true,
    "read-only Git allowlists must reject diff output-file options",
  );
  assert.equal(
    (await runner.emitToolCall(event("bash", { command: "git show --output /tmp/changed" })))?.block,
    true,
    "read-only Git allowlists must reject show output-file options",
  );
  assert.equal(
    (await runner.emitToolCall(event("bash", { command: "git log --output=/tmp/changed -1" })))?.block,
    true,
    "read-only Git allowlists must reject log output-file options",
  );
  assert.equal(
    (await runner.emitToolCall(event("bash", { command: "git diff --ext-diff" })))?.block,
    true,
    "read-only Git allowlists must reject external diff execution",
  );
  assert.equal(
    (await runner.emitToolCall(event("bash", { command: "find . -exec touch gate-bypass +" })))?.block,
    true,
    "read-only command allowlists must reject find command execution",
  );
  assert.equal(
    (await runner.emitToolCall(event("bash", { command: "find . -delete" })))?.block,
    true,
    "read-only command allowlists must reject destructive find commands",
  );
  assert.equal(
    (await runner.emitToolCall(event("bash", { command: "rg needle --pre=touch" })))?.block,
    true,
    "read-only command allowlists must reject ripgrep preprocessors",
  );

  const controlBash = event("bash", { command: "dijiang dispatch fix-the-gate" });
  assert.equal(await runner.emitToolCall(controlBash), undefined, "blocked Git Gate must allow standalone dispatch commands");

  const chainedControl = await runner.emitToolCall(event("bash", { command: "dijiang dispatch fix-the-gate && printf changed" }));
  assert.equal(chainedControl?.block, true, "shell chaining must not bypass the dispatch control-command allowlist");

  process.env.DIJIANG_PI_CONTRACT_STATE = workflowState("ready");
  process.env.DIJIANG_PI_CONTRACT_GIT_STATUS = " M src/main.rs";
  assert.equal(
    await runner.emitToolCall(event("write", { path: "src/main.rs", content: "x" })),
    undefined,
    "ready Git Gate must allow task-worktree writes",
  );

  const routeMessages = [];
  runner.bindCore(
    {
      sendMessage: (message) => routeMessages.push(message),
      sendUserMessage: () => {},
      appendEntry: () => {},
      setSessionName: () => {},
      getSessionName: () => undefined,
      setLabel: () => {},
      getActiveTools: () => [],
      getAllTools: () => [],
      setActiveTools: () => {},
      refreshTools: () => {},
      getCommands: () => [],
      setModel: async () => {},
      getThinkingLevel: () => undefined,
      setThinkingLevel: () => {},
    },
    {
      getModel: () => undefined,
      isIdle: () => true,
      isProjectTrusted: () => true,
      getSignal: () => undefined,
      abort: () => {},
      hasPendingMessages: () => false,
      shutdown: () => {},
      getContextUsage: () => undefined,
      compact: () => {},
      getSystemPrompt: () => "",
      getSystemPromptOptions: () => ({ cwd: tempRoot }),
    },
  );

  await runner.emitToolResult({ ...event("bash", { command: "find . -type f -print" }), isError: false });
  await runner.emitToolResult({ ...event("bash", { command: "cargo test -p dijiang-task" }), isError: false });
  assert.equal(routeMessages.length, 1, "only explicit successful validation commands may inject documentation routing");
  assert.match(routeMessages[0].content, /dj-output/);
  console.log("Pi extension tool_call contract passed");
} finally {
  process.env.PATH = originalPath;
  rmSync(tempRoot, { recursive: true, force: true });
}
