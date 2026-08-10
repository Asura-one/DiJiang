import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";

type ToolResultEvent = {
  toolName?: string;
  input?: { command?: string };
  content?: unknown;
  details?: unknown;
  isError?: boolean;
};

type WorkflowStatePayload = {
  activeTaskId?: string;
  activeTaskTitle?: string;
  activeTaskStatus?: string;
  capsule?: string;
  gitGateState?: string;
  expectedWorktreePath?: string;
  guidance?: string;
};

type WorkflowStateJson = {
  activeTask?: { id?: string; title?: string; status?: string };
  routeGate?: { capsule?: string };
  gitGate?: { state?: string; worktreePath?: string };
  guidance?: string;
};
const DEFAULT_CONTEXT_MAX_CHARS = 32768;
const MIN_CONTEXT_MAX_CHARS = 1024;
const MAX_CONTEXT_MAX_CHARS = 262144;

function contextMaxChars(): number {
  const raw = process.env.DIJIANG_CONTEXT_MAX_CHARS || "";
  const parsed = /^\d+$/.test(raw) ? Number(raw) : Number.NaN;
  return Number.isInteger(parsed) && parsed >= MIN_CONTEXT_MAX_CHARS && parsed <= MAX_CONTEXT_MAX_CHARS
    ? parsed
    : DEFAULT_CONTEXT_MAX_CHARS;
}

function limitContext(text: string, budget = contextMaxChars()): string {
  const chars = Array.from(text);
  if (chars.length <= budget) return text;
  const closing = "</dijiang-workflow-state>";
  const suffix = text.endsWith(closing) ? Array.from(closing) : [];
  let omitted = chars.length - budget;
  for (;;) {
    const marker = Array.from(`\n[…省略 ${omitted} 个字符…]\n`);
    const prefix = chars.slice(0, Math.max(0, budget - marker.length - suffix.length));
    const actualOmitted = chars.length - prefix.length - suffix.length;
    const output = [...prefix, ...marker, ...suffix].join("");
    if (Array.from(output).length <= budget && actualOmitted === omitted) return output;
    omitted = actualOmitted;
    if (Array.from(output).length <= budget) return output;
  }
}

function errorContext(message: string): string {
  const session =
    process.env.DIJIANG_CONTEXT_ID ||
    process.env.PI_SESSION_ID ||
    process.env.PI_SESSIONID ||
    "unknown";
  return [
    "<dijiang-workflow-state>",
    "平台: pi",
    `会话: ${session}`,
    `Hook 错误: ${message}`,
    "当前任务: unknown",
    "下一步: 在项目根目录运行 `dijiang workflow-state`，并确认 `dijiang` 已在 PATH 中。",
    "</dijiang-workflow-state>",
  ].join("\n");
}

function shellQuote(value: string): string {
  return `'${value.replace(/'/g, `'\\''`)}'`;
}

function contextKey(event?: unknown): string {
  const input = (event && typeof event === "object" ? event as Record<string, unknown> : {}) || {};
  const raw =
    process.env.DIJIANG_CONTEXT_ID ||
    process.env.PI_SESSION_ID ||
    process.env.PI_SESSIONID ||
    String(input.session_id || input.sessionId || input.sessionID || "");
  return (raw || "pi").replace(/[^A-Za-z0-9._-]+/g, "_").replace(/^[._-]+|[._-]+$/g, "").slice(0, 160) || "pi";
}

function commandHasDijiangContext(command: string): boolean {
  const trimmed = command.trim();
  return /^export\s+DIJIANG_CONTEXT_ID=/.test(trimmed) ||
    /^DIJIANG_CONTEXT_ID=/.test(trimmed) ||
    /^env\s+.*DIJIANG_CONTEXT_ID=/.test(trimmed);
}

function validationCommand(command: string): string | undefined {
  const normalized = command
    .replace(/^\s*(?:export\s+DIJIANG_CONTEXT_ID='[^']*';\s*)?/, "")
    .trim();
  const match = normalized.match(/^(?:cargo\s+(?:test|check|build)|npm\s+test|pnpm\s+test|yarn\s+test|vitest(?:\s|$)|tsc(?:\s|$)|(?:npm|pnpm|yarn)\s+run\s+(?:lint|typecheck|build))(?:\s|$)/i);
  return match?.[0].trim();
}

function exitCode(details: unknown): number | undefined {
  if (!details || typeof details !== "object") {
    return undefined;
  }
  const record = details as Record<string, unknown>;
  const code = record.code ?? record.exitCode ?? record.status;
  return typeof code === "number" ? code : undefined;
}
function failedToolResult(event: ToolResultEvent): boolean {
  if (event.isError) {
    return true;
  }
  const code = exitCode(event.details);
  return code !== undefined && code !== 0;
}

function routeMessage(route: string, reason: string, next: string): string {
  return [
    "<dijiang-route>",
    `路线: ${route}`,
    `原因: ${reason}`,
    `下一步: ${next}`,
    "</dijiang-route>",
  ].join("\n");
}

async function hasDirtyDiff(pi: ExtensionAPI): Promise<boolean> {
  try {
    const result = await pi.exec("git", ["status", "--porcelain"], { timeout: 3000 });
    return Boolean(result.stdout?.trim());
  } catch {
    return false;
  }
}

async function getWorkflowState(pi: ExtensionAPI): Promise<WorkflowStatePayload | null> {
  try {
    const result = await pi.exec("dijiang", ["workflow-state", "--json"], { timeout: 3000 });
    const payload = JSON.parse(result.stdout?.trim() || "{}") as { state?: WorkflowStateJson };
    const state = payload.state;
    if (!state) {
      return null;
    }
    return {
      activeTaskId: state.activeTask?.id,
      activeTaskTitle: state.activeTask?.title,
      activeTaskStatus: state.activeTask?.status,
      capsule: state.routeGate?.capsule,
      gitGateState: state.gitGate?.state,
      expectedWorktreePath: state.gitGate?.worktreePath,
      guidance: state.guidance,
    };
  } catch {
    return null;
  }
}

async function refreshStatusBar(ctx: { ui: ExtensionContext["ui"] }, pi: ExtensionAPI) {
  try {
    const state = await getWorkflowState(pi);
    if (!state) {
      ctx.ui.setStatus("dijiang-task", undefined);
      ctx.ui.setStatus("dijiang-capsule", undefined);
      return;
    }
    const title = state.activeTaskTitle || state.activeTaskId || "(none)";
    const capsule = state.capsule || "?";
    ctx.ui.setStatus("dijiang-task", `${title} [${capsule}]`);
    ctx.ui.setStatus("dijiang-capsule", `${capsule}`);
  } catch {
    // footer status is best-effort
  }
}

async function refreshWidget(ctx: { ui: ExtensionContext["ui"] }, pi: ExtensionAPI) {
  try {
    const state = await getWorkflowState(pi);
    if (!state) {
      ctx.ui.setWidget("dijiang", undefined);
      return;
    }
    const title = state.activeTaskTitle || state.activeTaskId || "无活跃任务";
    const capsule = state.capsule || "?";
    const gate = state.gitGateState || "-";
    ctx.ui.setWidget("dijiang", [
      `任务: ${title}  |  Capsule: ${capsule}  |  Gate: ${gate}`
    ]);
  } catch {
    // widget is best-effort
  }
}


async function dispatchContext(pi: ExtensionAPI, eventName: string, prompt: string): Promise<string | undefined> {
  try {
    const result = await pi.exec("dijiang", [
      "dispatch",
      prompt,
      "--json",
      "--hook-event",
      eventName,
    ]);
    const payload = JSON.parse(result.stdout?.trim() || "{}");
    const rawContext = payload.additionalContext;
    const context = typeof rawContext === "string" ? limitContext(rawContext.trim()) : "";
    if (context) {
      pi.appendEntry("dijiang_dispatch", { context, eventName });
      return context;
    }
  } catch (error) {
    const context = errorContext(error instanceof Error ? error.message : String(error));
    pi.appendEntry("dijiang_dispatch", { context, eventName });
    return context;
  }
  return undefined;
}
function hasShellOperators(command: string): boolean {
  return /[;&|><`$()\r\n]/.test(command);
}

function isDijiangControlCommand(command: string): boolean {
  return !hasShellOperators(command) && /^\s*dijiang\s+(?:dispatch|workflow-state)(?:\s|$)/i.test(command);
}




function hasGitWriteOption(command: string): boolean {
  return /\bgit\b[^\r\n]*(?:\s-o(?:\s|$)|\s--output(?:=|\s)|\s--ext-diff(?:\s|$))|\bgit\s+branch\b[^\r\n]*\s--edit-description(?:\s|$)/i.test(command);
}

function isReadOnlyCommand(command: string): boolean {
  if (hasShellOperators(command) || hasGitWriteOption(command) || /^\s*rg\b[^\r\n]*\s--pre(?:=|\s|$)/i.test(command)) {
    return false;
  }
  return /^\s*(?:pwd|git\s+(?:status|diff|log|show|branch(?:\s+--list)?|worktree\s+list|rev-parse|ls-files|remote\s+-v|config\s+--get|grep)(?:\s|$)|(?:rg|grep|ls|cat|head|tail)\b)/i.test(command);
}

function requiresWorktreeGate(toolName?: string): boolean {
  return ["bash", "write", "edit", "apply_patch", "replace"].includes(toolName || "");
}


function blockedWorktreeReason(worktreePath?: string): string {
  const next = worktreePath
    ? `请从任务 worktree 重启 Pi：\`cd ${worktreePath} && pi\`。`
    : "先建立 Git 基线并重新运行 `dijiang dispatch <request>`。";
  return `DiJiang Git Gate 已阻止在当前目录执行可能修改工作区的工具调用；${next}`;
}
async function injectWorkflowState(pi: ExtensionAPI, eventName: string) {
  try {
    const result = await pi.exec("dijiang", [
      "workflow-state",
      "--hook-event",
      eventName,
    ]);
    const context = result.stdout?.trim();
    if (context) {
      pi.appendEntry("dijiang_workflow_state", { context, eventName });
    }
  } catch (error) {
    pi.appendEntry("dijiang_workflow_state", {
      context: errorContext(error instanceof Error ? error.message : String(error)),
      eventName,
    });
  }
}

export default function (pi: ExtensionAPI) {
  let lastDocsInjection = "";

  pi.registerCommand("dijiang", {
    description: "Show DiJiang task status, phase, and capsule info",
    handler: async (_args, ctx) => {
      const state = await getWorkflowState(pi);
      if (!state) {
        ctx.ui.notify("DiJiang: 未检测到工作流状态", "warning");
        return;
      }
      const title = state.activeTaskTitle || state.activeTaskId || "(none)";
      const capsule = state.capsule || "?";
      const gate = state.gitGateState || "-";
      await refreshWidget(ctx, pi);
      await refreshStatusBar(ctx, pi);
      ctx.ui.notify(`任务: ${title} | Capsule: ${capsule} | Gate: ${gate}`, "info");
    },
  });

  async function maybeDispatchFromPrompt(eventName: string, prompt?: string) {
    const text = prompt?.trim();
    if (!text || text.startsWith("/")) {
      return undefined;
    }

    const context = await dispatchContext(pi, eventName, text);
    if (!context) {
      return undefined;
    }

    return {
      message: {
        customType: "dijiang_dispatch",
        content: context,
        display: false,
      },
    };
  }
  pi.on("before_agent_start", async (event) => {
    return maybeDispatchFromPrompt("before_agent_start", event.prompt);
  });


  pi.on("tool_call", async (event) => {
    const ev = event as ToolResultEvent;
    if (requiresWorktreeGate(ev.toolName)) {
      const state = await getWorkflowState(pi);
      if (state?.gitGateState === "blocked") {
        const command = ev.input?.command;
        const isControlCommand = typeof command === "string" && isDijiangControlCommand(command);
        const isReadOnly = typeof command === "string" && isReadOnlyCommand(command);
        if (!isControlCommand && !isReadOnly) {
          return {
            block: true,
            reason: blockedWorktreeReason(state.expectedWorktreePath),
          };
        }
      }
    }

    if (
      ev.toolName === "bash" &&
      ev.input &&
      typeof ev.input.command === "string" &&
      !commandHasDijiangContext(ev.input.command)
    ) {
      ev.input.command = `export DIJIANG_CONTEXT_ID=${shellQuote(contextKey(event))}; ${ev.input.command}`;
    }
  });

  pi.on("tool_result", async (event) => {
    const ev = event as ToolResultEvent;
    const command = ev.input?.command ?? "";
    if (ev.toolName !== "bash" || !command) {
      return;
    }

    const validation = validationCommand(command);
    if (!validation || failedToolResult(ev)) {
      return;
    }


    if (await hasDirtyDiff(pi)) {
      const key = `${contextKey(event)}:${command}:docs`;
      if (key !== lastDocsInjection) {
        lastDocsInjection = key;
        const context = routeMessage(
          "dj-output",
          `validation/check passed with dirty git diff: ${command}`,
          "sync task artifacts and relevant docs/spec before finish-work.",
        );
        pi.appendEntry("dijiang_route", { route: "dj-output", command, context });
        pi.sendMessage({
          customType: "dijiang_route",
          content: context,
          display: false,
          details: { route: "dj-output", command },
        }, { deliverAs: "steer" });
      }
    }
  });

  pi.on("session_start", async () => {
    await injectWorkflowState(pi, "session_start");
  });

  pi.on("session_shutdown", async () => {
    await injectWorkflowState(pi, "session_shutdown");
  });
}
