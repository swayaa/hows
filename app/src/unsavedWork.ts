import type { RecorderPhase } from "./chrome";

/** Everything the app, the tray or a hotkey may ask to do with the guide on screen. */
export type Intent = "start" | "pause" | "resume" | "stop" | "discard" | "open" | "quit";

export type GuideWork = {
  dirty: boolean;
  phase: RecorderPhase;
  /** Steps that exist only in the running recording. */
  capturedSteps: number;
};

const DROPS_EDITOR: Record<Intent, boolean> = {
  start: true,
  pause: false,
  resume: false,
  stop: false,
  discard: true,
  open: true,
  quit: true,
};

/** Quit also drops a recording that already has steps. */
function quitDropsWork(work: GuideWork): boolean {
  const recording =
    (work.phase === "recording" || work.phase === "paused") && work.capturedSteps > 0;
  const editor = work.phase === "reviewing" && work.dirty;
  return recording || editor;
}

/**
 * Asks only when `intent` would throw away an edited guide or a recording
 * that already has steps. Resolves to whether the intent may run.
 */
export async function confirmIntent(
  intent: Intent,
  work: GuideWork,
  ask: (intent: Intent) => Promise<boolean>,
): Promise<boolean> {
  const drops =
    intent === "quit"
      ? quitDropsWork(work)
      : DROPS_EDITOR[intent] && work.dirty && work.phase === "reviewing";
  if (!drops) return true;
  return ask(intent);
}

/** Whether `text` differs from what the selected step already says. */
export function stepTextChanged(step: { text: string }, text: string): boolean {
  return text !== step.text;
}

/** Enter keeps the step text. Shift+Enter stays a new line. */
export function stepTextConfirms(key: {
  key: string;
  shiftKey: boolean;
  isComposing: boolean;
}): boolean {
  return key.key === "Enter" && !key.shiftKey && !key.isComposing;
}
