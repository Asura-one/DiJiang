import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export default function (pi: ExtensionAPI) {
  pi.on("session_start", async () => {
    try {
      const result = await pi.exec("dijiang", ["workflow-state"]);
      const context = result.stdout?.trim();
      if (context) {
        pi.appendEntry("dijiang_workflow_state", { context });
      }
    } catch {
      // dijiang CLI not available or project not initialized
    }
  });
}
}
