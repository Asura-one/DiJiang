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
  pi.on("session_start", async () => {
    await injectWorkflowState(pi, "session_start");
  });

  pi.on("session_shutdown", async () => {
    await injectWorkflowState(pi, "session_shutdown");
  });
}
