/* global process */
import { execFileSync } from "node:child_process";

export default async ({ directory }) => {
  return {
    "chat.message": async (_input, output) => {
      const parts = output?.parts || [];
      let context = "";

      try {
        context = execFileSync("dijiang", ["workflow-state"], {
          cwd: directory,
          encoding: "utf8",
          timeout: 10000,
        }).trim();
      } catch {
        context = [
          "<dijiang-workflow-state>",
          "Active task: unknown",
          "Next: run `dijiang workflow-state` from the project root.",
          "</dijiang-workflow-state>",
        ].join("\n");
      }

      if (!context) {
        return;
      }

      const textPartIndex = parts.findIndex(
        (part) => part.type === "text" && part.text !== undefined
      );

      if (textPartIndex !== -1) {
        parts[textPartIndex].text = context + "\n\n---\n\n" + (parts[textPartIndex].text || "");
      } else {
        parts.unshift({ type: "text", text: context });
      }
    },
  };
};
