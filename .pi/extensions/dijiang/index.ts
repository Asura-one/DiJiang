import { defineExtension } from "pi";
import { readFileSync } from "fs";
import { join } from "path";

export default defineExtension({
  name: "dijiang",
  "session:start": async (ctx) => {
    const cwd = process.cwd();

    // Read active task context via dijiang CLI
    try {
      const { execSync } = require("child_process");
      const result = execSync(`dijiang task current`, {});
      const taskPath = result.toString().trim();
      if (taskPath && taskPath !== "No active task") {
        ctx.setVar("activeTask", taskPath);
      }
    } catch {}

    // Read spec index
    try {
      const specIndex = join(cwd, ".trellis/spec/index.md");
      const content = readFileSync(specIndex, "utf-8");
      ctx.setVar("specIndex", content);
    } catch {}
  },
});