import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { readFileSync } from "fs";
import { join } from "path";

export default function (pi: ExtensionAPI) {
  pi.on("session_start", async (_event, ctx) => {
    const cwd = ctx.cwd;

    // Inject active task context into session
    try {
      const result = await pi.exec("dijiang", ["task", "current"]);
      const taskPath = result.stdout?.trim();
      if (taskPath && taskPath !== "No active task") {
        pi.appendEntry("task_context", { activeTask: taskPath });
      }
    } catch {
      // dijiang CLI not available or no active task
    }

    // Inject spec index into session
    try {
      const specIndex = join(cwd, ".trellis/spec/index.md");
      const content = readFileSync(specIndex, "utf-8");
      pi.appendEntry("spec_context", { specIndex: content });
    } catch {
      // No spec index found
    }
  });
}
