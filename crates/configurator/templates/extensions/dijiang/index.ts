import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

type ToolResultEvent = {
  toolName?: string;
  input?: { command?: string };
  content?: unknown;
  details?: unknown;
  isError?: boolean;
};

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

function isValidationCommand(command: string): boolean {
  return /\b(test|typecheck|lint|build|check|cargo\s+test|cargo\s+check|pnpm\s+test|npm\s+test|vitest|tsc)\b/i.test(command);
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
    const context = payload.additionalContext?.trim();
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
  let lastHuntInjection = "";
  let lastDocsInjection = "";

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

  pi.on("user_prompt_submit", async (event) => {
    return maybeDispatchFromPrompt("user_prompt_submit", event.prompt);
  });

  pi.on("tool_call", (event) => {
    const ev = event as { toolName?: string; input?: { command?: string } };
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

    if (failedToolResult(ev)) {
      const key = `${contextKey(event)}:${command}:hunt`;
      if (key !== lastHuntInjection) {
        lastHuntInjection = key;
        const context = routeMessage(
          "dj-hunt",
          `bash command failed: ${command}`,
          "stop normal implementation, diagnose root cause, fix, then return to dj-check.",
        );
        pi.appendEntry("dijiang_route", { route: "dj-hunt", command, context });
        pi.sendMessage({
          customType: "dijiang_route",
          content: context,
          display: false,
          details: { route: "dj-hunt", command },
        }, { deliverAs: "steer" });
      }
      return;
    }

    if (!isValidationCommand(command)) {
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
