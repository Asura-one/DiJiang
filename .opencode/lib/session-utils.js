/**
 * DiJiang session utilities for OpenCode plugins.
 */
export function buildSessionContext(ctx, input) {
  return `DiJiang project at ${ctx.directory}`;
}
