<script lang="ts">
  import type { Messages } from "./locales";
  import ResetButton from "./ResetButton.svelte";
  import {
    defaultChange,
    exampleFileName,
    exampleTitle,
    formatSettingNumber,
    isDefault,
    parseSettingNumber,
    type NumericField,
    type SettingChange,
    type SettingsDefaults,
    type SettingsView,
    type TunableField,
  } from "./settingsUi";
  import { fillTemplate } from "./template";

  let {
    copy,
    localeCode,
    view,
    defaults,
    onChange,
    onResetAll,
  }: {
    copy: Messages;
    /** UI language; numbers are shown and typed in its notation. */
    localeCode: string;
    view: SettingsView;
    /** Backend defaults and limits; `null` until they are loaded. */
    defaults: SettingsDefaults | null;
    onChange: (change: SettingChange) => void;
    onResetAll: () => void;
  } = $props();

  /** A number setting; `scale` converts the stored unit to the shown one (ms to s). */
  type NumberRow = {
    field: NumericField;
    label: string;
    unit: string;
    scale: number;
    integer: boolean;
  };

  const numberRows = $derived<NumberRow[]>([
    { field: "typing_pause_ms", label: copy.typingPause, unit: copy.unitSeconds, scale: 1000, integer: true },
    { field: "scroll_pause_ms", label: copy.scrollPause, unit: copy.unitSeconds, scale: 1000, integer: true },
    { field: "recent_limit", label: copy.recentLimit, unit: "", scale: 1, integer: true },
    { field: "pdf_margin_mm", label: copy.pdfMargin, unit: copy.unitMillimeters, scale: 1, integer: true },
    { field: "annotation_stroke", label: copy.annotationStroke, unit: copy.unitPixels, scale: 1, integer: false },
  ]);

  const sampleTitle = $derived(exampleTitle(view, copy.sampleApp));
  const sampleFileName = $derived(exampleFileName(view, sampleTitle, copy.sampleApp));

  function shownValue(row: NumberRow, stored: number): string {
    return formatSettingNumber(stored / row.scale, localeCode);
  }

  /** Clamps to the backend limits, stores the value and rewrites the field in the UI notation. */
  function commitNumber(row: NumberRow, input: HTMLInputElement, shown: number | null) {
    if (shown === null) {
      input.value = shownValue(row, view[row.field]);
      return;
    }
    const limit = defaults?.limits[row.field];
    const scaled = shown * row.scale;
    const clamped = limit ? Math.min(limit.max, Math.max(limit.min, scaled)) : scaled;
    const stored = row.integer ? Math.round(clamped) : Math.round(clamped * 1000) / 1000;
    input.value = shownValue(row, stored);
    onChange({ field: row.field, value: stored } as SettingChange);
  }

  function numberChange(row: NumberRow, event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    commitNumber(row, input, parseSettingNumber(input.value));
  }

  /** Arrow keys step like a native number field. */
  function numberKeydown(row: NumberRow, event: KeyboardEvent) {
    if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return;
    event.preventDefault();
    const input = event.currentTarget as HTMLInputElement;
    const step = (defaults?.limits[row.field]?.step ?? 1) / row.scale;
    const current = parseSettingNumber(input.value) ?? view[row.field] / row.scale;
    commitNumber(row, input, current + (event.key === "ArrowUp" ? step : -step));
  }

  function textChange(field: "title_template" | "file_name_template", event: Event) {
    onChange({ field, value: (event.currentTarget as HTMLInputElement).value });
  }

  function reset(field: TunableField) {
    const next = defaultChange(defaults, field);
    if (next) onChange(next);
  }
</script>

<details class="sl-settings-block sl-settings-advanced" data-section="advanced">
  <summary class="sl-settings-group-label">{copy.groupAdvanced}</summary>
  <p class="sl-settings-help">{copy.advancedHint}</p>
  <div class="sl-settings-rows">
    {#each numberRows as row (row.field)}
      {@const limit = defaults?.limits[row.field]}
      <div class="sl-settings-row">
        <label for={`setting-${row.field}`}>{row.label}</label>
        <span class="sl-settings-control">
          <ResetButton
            {copy}
            label={row.label}
            changed={!isDefault(view, defaults, row.field)}
            onReset={() => reset(row.field)}
          />
          <input
            id={`setting-${row.field}`}
            class="sl-settings-number"
            type="text"
            role="spinbutton"
            inputmode={row.integer && row.scale === 1 ? "numeric" : "decimal"}
            data-setting={row.field}
            value={shownValue(row, view[row.field])}
            aria-valuenow={view[row.field] / row.scale}
            aria-valuemin={limit ? limit.min / row.scale : undefined}
            aria-valuemax={limit ? limit.max / row.scale : undefined}
            onchange={(event) => numberChange(row, event)}
            onkeydown={(event) => numberKeydown(row, event)}
          />
          <span class="sl-settings-unit">{row.unit}</span>
        </span>
      </div>
    {/each}

    <div class="sl-settings-row sl-settings-row-stack">
      <span class="sl-settings-row-head">
        <label for="setting-title_template">{copy.titleTemplate}</label>
        <ResetButton
          {copy}
          label={copy.titleTemplate}
          changed={!isDefault(view, defaults, "title_template")}
          onReset={() => reset("title_template")}
        />
      </span>
      <input
        id="setting-title_template"
        class="sl-settings-text"
        type="text"
        data-setting="title_template"
        value={view.title_template}
        placeholder={view.language_title_template}
        onchange={(event) => textChange("title_template", event)}
      />
      <p class="sl-settings-help">{copy.titleTemplateHelp}</p>
      <p class="sl-settings-example">{fillTemplate(copy.templateExample, { example: sampleTitle })}</p>
    </div>

    <div class="sl-settings-row sl-settings-row-stack">
      <span class="sl-settings-row-head">
        <label for="setting-file_name_template">{copy.fileNameTemplate}</label>
        <ResetButton
          {copy}
          label={copy.fileNameTemplate}
          changed={!isDefault(view, defaults, "file_name_template")}
          onReset={() => reset("file_name_template")}
        />
      </span>
      <input
        id="setting-file_name_template"
        class="sl-settings-text"
        type="text"
        data-setting="file_name_template"
        value={view.file_name_template}
        onchange={(event) => textChange("file_name_template", event)}
      />
      <p class="sl-settings-help">{copy.fileNameTemplateHelp}</p>
      <p class="sl-settings-example">{fillTemplate(copy.templateExample, { example: `${sampleFileName}.html` })}</p>
    </div>
  </div>
  <button class="sl-btn sl-btn-outlined sl-settings-reset-all" type="button" data-action="reset-advanced" onclick={onResetAll}>
    {copy.resetAdvanced}
  </button>
</details>
