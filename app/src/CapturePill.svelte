<script lang="ts">
  import "./styles/capture-pill.css";
  import Pause from "@lucide/svelte/icons/pause";
  import Play from "@lucide/svelte/icons/play";
  import Square from "@lucide/svelte/icons/square";
  import { invoke } from "@tauri-apps/api/core";
  import { ariaKeyShortcuts, type RecorderPhase } from "./chrome";
  import type { Messages } from "./locales";

  let {
    copy,
    recorderState,
    stepLabel,
    recordHotkey,
    pauseHotkey,
    onCommand,
  }: {
    copy: Messages;
    recorderState: RecorderPhase;
    /** "3 Schritte" for the live counter. */
    stepLabel: string;
    /** Chords as stored in the settings, for example "Ctrl+Shift+F9". */
    recordHotkey: { chord: string; label: string };
    pauseHotkey: { chord: string; label: string };
    onCommand: (name: "pause" | "resume" | "stop") => void;
  } = $props();

  const paused = $derived(recorderState === "paused");
  const statusText = $derived(paused ? copy.capturePaused : copy.captureLive);

  let pill: HTMLDivElement | undefined = $state();

  /** The window shrinks to the pill, so every size change is reported. */
  $effect(() => {
    if (!pill) return;
    const observer = new ResizeObserver(() => {
      if (!pill) return;
      const { width, height } = pill.getBoundingClientRect();
      void invoke("fit_capture_window", { width: Math.ceil(width), height: Math.ceil(height) });
    });
    observer.observe(pill);
    return () => observer.disconnect();
  });
</script>

<div
  bind:this={pill}
  class="sl-capture-pill"
  role="status"
  aria-live="polite"
  data-tauri-drag-region
>
  <span class="sl-rec-dot" class:paused title={statusText} aria-hidden="true"></span>
  <span class="sl-capture-status" class:sl-sr-only={!paused} data-tauri-drag-region>{statusText}</span>
  <span class="sl-capture-count" data-tauri-drag-region>{stepLabel}</span>
  <span class="sl-pill-sep" aria-hidden="true"></span>
  <button
    class="sl-pill-btn"
    type="button"
    data-action={paused ? "capture-resume" : "capture-pause"}
    aria-keyshortcuts={ariaKeyShortcuts(pauseHotkey.chord)}
    onclick={() => onCommand(paused ? "resume" : "pause")}
  >
    {#if paused}<Play class="sl-icon" />{copy.resume}{:else}<Pause class="sl-icon" />{copy.pause}{/if}
    <kbd class="sl-pill-kbd" data-part="hotkey-pause">{pauseHotkey.label}</kbd>
  </button>
  <button
    class="sl-pill-btn sl-pill-btn-danger"
    type="button"
    data-action="capture-stop"
    aria-keyshortcuts={ariaKeyShortcuts(recordHotkey.chord)}
    onclick={() => onCommand("stop")}
  >
    <Square class="sl-icon" />{copy.stop}
    <kbd class="sl-pill-kbd" data-part="hotkey-stop">{recordHotkey.label}</kbd>
  </button>
</div>
