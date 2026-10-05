<script lang="ts">
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import type { Messages } from "./locales";
  import { fillTemplate } from "./template";

  let {
    copy,
    label,
    changed,
    onReset,
  }: {
    copy: Messages;
    /** Name of the setting, read out as "Reset <label>". */
    label: string;
    /** Only a changed value offers a reset. */
    changed: boolean;
    onReset: () => void;
  } = $props();
</script>

{#if changed}
  <button
    class="sl-btn sl-btn-small sl-reset"
    type="button"
    data-action="reset-setting"
    aria-label={fillTemplate(copy.resetValueLabel, { setting: label })}
    onclick={onReset}
  >
    <RotateCcw class="sl-icon" />{copy.resetValue}
  </button>
{/if}
