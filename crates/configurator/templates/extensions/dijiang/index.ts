import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

function errorContext(message: string): string {
  const session =
    process.env.DIJIANG_CONTEXT_ID ||
    process.env.PI_SESSION_ID ||
    process.env.PI_SESSIONID ||
    "unknown";
  return [
    "<dijiang-workflow-state>",
    "Platform: pi",
    `Session hint: ${session}`,
    `Hook error: ${message}`,
    "Active task: unknown",
    "Next: run `dijiang workflow-state` from the project root and check that `dijiang` is on PATH.",
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
  pi.on("before_agent_start", async (event) => {
    const text = event.prompt?.trim();
    if (!text || text.startsWith("/")) {
      return;
    }

    const context = await dispatchContext(pi, "before_agent_start", text);
    if (!context) {
      return;
    }

    return {
      message: {
        customType: "dijiang_dispatch",
        content: context,
        display: false,
      },
    };
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

  pi.on("session_start", async () => {
    await injectWorkflowState(pi, "session_start");
  });

  pi.on("session_shutdown", async () => {
    await injectWorkflowState(pi, "session_shutdown");
  });
}
