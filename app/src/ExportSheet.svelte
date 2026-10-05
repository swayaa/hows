<script lang="ts">
  import "./styles/export-sheet.css";
  import X from "@lucide/svelte/icons/x";
  import type { Messages } from "./locales";
  import { shortenPath, type ExportFormat } from "./settingsUi";

  let {
    copy,
    format = $bindable(),
    openAfter = $bindable(),
    targetFolder,
    targetSynced,
    onChangeFolder,
    onRun,
    onClose,
  }: {
    copy: Messages;
    format: ExportFormat;
    openAfter: boolean;
    /** Resolved folder the selected format writes to. */
    targetFolder: string;
    /** The target folder is under OneDrive. Shown before the first export. */
    targetSynced: boolean;
    onChangeFolder: () => void;
    onRun: () => void;
    onClose: () => void;
  } = $props();

  const rows = $derived([
    { value: "html" as const, label: copy.exportHtml, badge: copy.exportHtmlBadge, accent: true },
    { value: "pdf" as const, label: copy.exportPdf, badge: copy.exportPdfBadge, accent: false },
    { value: "steps" as const, label: copy.exportSteps, badge: copy.exportStepsBadge, accent: false },
  ]);

  $effect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<div
  class="sl-sheet-backdrop"
  role="presentation"
  onclick={(event) => {
    if (event.target === event.currentTarget) onClose();
  }}
>
  <div class="sl-sheet" role="dialog" aria-modal="true" aria-label={copy.exportTitle}>
    <div class="sl-sheet-head">
      <h2>{copy.exportTitle}</h2>
      <button class="sl-icon-btn" type="button" aria-label={copy.closeSheet} data-action="close-export" onclick={onClose}>
        <X class="sl-icon" />
      </button>
    </div>

    <div class="sl-export-formats" role="radiogroup" aria-label={copy.exportTitle}>
      {#each rows as row (row.value)}
        <button
          type="button"
          class="sl-export-row"
          class:selected={format === row.value}
          role="radio"
          aria-checked={format === row.value}
          onclick={() => (format = row.value)}
        >
          <span class="sl-export-row-main">
            <span class="sl-export-radio" aria-hidden="true"></span>
            <span class="sl-export-row-label">{row.label}</span>
          </span>
          <span class="sl-export-badge" class:sl-export-badge-accent={row.accent}>
            {row.badge}
          </span>
        </button>
      {/each}
    </div>

    <div class="sl-export-target">
      <span class="sl-export-target-label">{copy.exportTarget}</span>
      <span class="sl-export-target-path" title={targetFolder}>
        {shortenPath(targetFolder, 44)}
      </span>
      <button class="sl-btn sl-btn-small" type="button" onclick={onChangeFolder}>
        {copy.exportChangeFolder}
      </button>
    </div>
    {#if targetSynced}
      <p class="sl-export-warning" role="alert">{copy.exportSyncedWarning}</p>
    {/if}

    <label class="sl-export-checkbox">
      <input type="checkbox" bind:checked={openAfter} />
      <span>{copy.exportOpenAfter}</span>
    </label>

    <button class="sl-btn sl-btn-primary sl-export-primary" type="button" data-action="run-export" onclick={onRun}>
      {copy.exportPrimary}
    </button>

    <p class="sl-sheet-footer">{copy.exportFooter}</p>
  </div>
</div>
