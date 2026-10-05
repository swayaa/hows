<script lang="ts">
  import "./styles/settings-page.css";
  import { invoke } from "@tauri-apps/api/core";
  import { UI_LOCALES, uiLocale, type Messages } from "./locales";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import AdvancedSettings from "./AdvancedSettings.svelte";
  import { BRANDS, findBrand } from "./brand";
  import { asCommandError, reportError } from "./commandError";
  import ResetButton from "./ResetButton.svelte";
  import {
    asExportFormat,
    asThemePreference,
    chordFromKeyboardEvent,
    defaultChange,
    formatHotkeyLabel,
    isDefault,
    localizedName,
    shortenPath,
    SYSTEM_LANGUAGE,
    type SettingChange,
    type SettingsDefaults,
    type SettingsView,
    type TunableField,
  } from "./settingsUi";

  let {
    copy,
    localeCode,
    view = $bindable(),
    defaults,
    onClose,
    onError,
  }: {
    copy: Messages;
    /** Resolved UI locale, one of `UI_LOCALES`. */
    localeCode: string;
    /** Backend settings; every change here is persisted and written back. */
    view: SettingsView;
    /** Backend defaults and limits; `null` until they are loaded. */
    defaults: SettingsDefaults | null;
    onClose: () => void;
    onError: (message: string) => void;
  } = $props();

  /** Accent of the chosen preset in the light theme, as the color picker shows it. */
  const presetAccent = $derived(findBrand(view.brand).light.accent);

  /** Persist one value; the backend clamps it and answers with the new view. */
  async function change(next: SettingChange) {
    try {
      view = await invoke<SettingsView>("change_setting", { change: next });
    } catch (error) {
      onError(reportError(error, copy));
    }
  }

  async function resetAdvanced() {
    try {
      view = await invoke<SettingsView>("reset_advanced_settings");
    } catch (error) {
      onError(reportError(error, copy));
    }
  }

  function reset(field: TunableField) {
    const next = defaultChange(defaults, field);
    if (next) void change(next);
  }

  type HotkeySlot = "record" | "pause";
  type FolderKind = "export" | "guides";

  let hotkeyListening = $state<HotkeySlot | null>(null);
  let hotkeyError = $state<HotkeySlot | null>(null);
  let hotkeyAlert = $state("");

  const hotkeyRows = $derived([
    {
      which: "record" as const,
      label: copy.hotkeyRecord,
      shown: formatHotkeyLabel(view.hotkey, copy.keyNames),
    },
    {
      which: "pause" as const,
      label: copy.hotkeyPause,
      shown: formatHotkeyLabel(view.pause_hotkey, copy.keyNames),
    },
  ]);
  const folderRows = $derived([
    {
      which: "export" as const,
      label: copy.defaultFolder,
      path: view.export_folder_display,
      shown: shortenPath(view.export_folder_display),
      fallback: view.export_folder_fallback,
    },
    {
      which: "guides" as const,
      label: copy.guidesFolder,
      path: view.guides_folder_display,
      shown: shortenPath(view.guides_folder_display),
      fallback: view.guides_folder_fallback,
    },
  ]);

  function toggleHotkeyListening(which: HotkeySlot) {
    hotkeyError = null;
    hotkeyAlert = "";
    hotkeyListening = hotkeyListening === which ? null : which;
  }

  $effect(() => {
    if (!hotkeyListening) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        hotkeyListening = null;
        return;
      }
      const chord = chordFromKeyboardEvent(event);
      if (!chord) return;
      event.preventDefault();
      event.stopPropagation();
      const target = hotkeyListening;
      hotkeyListening = null;
      if (target) void commitHotkey(target, chord);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  async function commitHotkey(which: HotkeySlot, chord: string) {
    hotkeyError = null;
    hotkeyAlert = "";
    const other = which === "record" ? view.pause_hotkey : view.hotkey;
    if (chord.trim().toLowerCase() === other.trim().toLowerCase()) {
      hotkeyError = which;
      hotkeyAlert = copy.hotkeyConflict;
      return;
    }
    try {
      const unregistered_hotkeys = view.unregistered_hotkeys.filter((slot) => slot !== which);
      if (which === "record") {
        await invoke("set_hotkey", { hotkey: chord });
        view = { ...view, hotkey: chord, unregistered_hotkeys };
      } else {
        await invoke("set_pause_hotkey", { hotkey: chord });
        view = { ...view, pause_hotkey: chord, unregistered_hotkeys };
      }
    } catch (error) {
      hotkeyError = which;
      hotkeyAlert =
        asCommandError(error).code === "hotkey_invalid"
          ? copy.errors.hotkey_invalid
          : copy.hotkeyConflict;
    }
  }

  /** Persist one setting after the view was updated locally. */
  async function setSetting(command: string, args: Record<string, unknown>) {
    try {
      await invoke(command, args);
    } catch (error) {
      onError(reportError(error, copy));
    }
  }

  async function onLanguageChange(event: Event) {
    const picked = (event.currentTarget as HTMLSelectElement).value;
    const language = picked === SYSTEM_LANGUAGE ? picked : uiLocale(picked).code;
    view = { ...view, language };
    try {
      view = await invoke<SettingsView>("set_language", { language });
    } catch (error) {
      onError(reportError(error, copy));
    }
  }

  function onThemeChange(event: Event) {
    const theme = asThemePreference((event.currentTarget as HTMLSelectElement).value);
    view = { ...view, theme };
    void setSetting("set_theme", { theme });
  }

  function onExportFormatChange(event: Event) {
    const format = asExportFormat((event.currentTarget as HTMLSelectElement).value);
    view = { ...view, export_format: format };
    void setSetting("set_export_format", { format });
  }

  function onOpenAfterChange(event: Event) {
    const open = (event.currentTarget as HTMLInputElement).checked;
    view = { ...view, open_after_export: open };
    void setSetting("set_open_after_export", { open });
  }

  async function browseFolder(which: FolderKind) {
    const title = which === "export" ? copy.defaultFolder : copy.guidesFolder;
    try {
      const picked = await invoke<string | null>("pick_folder", { title });
      if (!picked) return;
      await commitFolder(which, picked);
    } catch (error) {
      onError(reportError(error, copy));
    }
  }

  async function commitFolder(which: FolderKind, path: string) {
    const command = which === "export" ? "set_default_export_folder" : "set_guides_folder";
    try {
      view = await invoke<SettingsView>(command, { path });
    } catch (error) {
      // `invalid_folder` carries the default folder in `path`; keep the previous value, show the default.
      const failure = asCommandError(error);
      const fallback = failure.code === "invalid_folder" ? (failure.path ?? "") : "";
      if (!fallback) {
        onError(reportError(error, copy));
        return;
      }
      view =
        which === "export"
          ? {
              ...view,
              export_folder_fallback: true,
              export_folder_display: fallback || view.export_folder_display,
            }
          : {
              ...view,
              guides_folder_fallback: true,
              guides_folder_display: fallback || view.guides_folder_display,
            };
      onError(copy.pathInvalid);
    }
  }
</script>

<section class="sl-settings">
  <header class="sl-settings-head">
    <button class="sl-back" type="button" data-action="close-settings" onclick={onClose}>
      <ArrowLeft class="sl-icon" />{copy.backGeneric}
    </button>
    <div class="sl-settings-titles">
      <h1 class="sl-settings-title">{copy.settingsTitle}</h1>
      <p class="sl-settings-rare">{copy.settingsRareCue}</p>
    </div>
  </header>

  <div class="sl-settings-body">
    <section class="sl-settings-block" aria-labelledby="settings-group-capture">
      <h2 id="settings-group-capture" class="sl-settings-group-label">{copy.groupCapture}</h2>
      <div class="sl-settings-rows">
        {#each hotkeyRows as row (row.which)}
          <div class="sl-settings-row">
            <span>{row.label}</span>
            <span class="sl-settings-control">
              <button
                type="button"
                class="sl-hotkey-capture"
                class:listening={hotkeyListening === row.which}
                aria-label={row.label}
                onclick={() => toggleHotkeyListening(row.which)}
              >
                {hotkeyListening === row.which ? copy.hotkeyListening : row.shown}
              </button>
            </span>
          </div>
          {#if hotkeyError === row.which}
            <p class="sl-settings-error" role="alert">{hotkeyAlert}</p>
          {:else if view.unregistered_hotkeys.includes(row.which)}
            <p class="sl-settings-error" role="alert">{copy.hotkeyTakenAtStartup}</p>
          {/if}
        {/each}
      </div>
    </section>

    <section class="sl-settings-block" aria-labelledby="settings-group-export">
      <h2 id="settings-group-export" class="sl-settings-group-label">{copy.groupExport}</h2>
      <div class="sl-settings-rows">
        <label class="sl-settings-row" for="export-format-select">
          <span>{copy.exportDefault}</span>
          <span class="sl-settings-control">
            <select
              id="export-format-select"
              value={asExportFormat(view.export_format)}
              onchange={onExportFormatChange}
              aria-label={copy.exportDefault}
            >
              <option value="html">{copy.exportFormatHtml} · {copy.exportRecommended}</option>
              <option value="pdf">{copy.exportFormatPdf}</option>
              <option value="steps">{copy.exportFormatSteps}</option>
            </select>
          </span>
        </label>

        <label class="sl-settings-row" for="open-after-check">
          <span>{copy.openAfterExport}</span>
          <span class="sl-settings-control">
            <input
              id="open-after-check"
              type="checkbox"
              checked={view.open_after_export}
              onchange={onOpenAfterChange}
            />
          </span>
        </label>

        <div class="sl-settings-row">
          <label for="export-credit-check">{copy.exportCredit}</label>
          <span class="sl-settings-control">
            <ResetButton
              {copy}
              label={copy.exportCredit}
              changed={!isDefault(view, defaults, "export_credit")}
              onReset={() => reset("export_credit")}
            />
            <input
              id="export-credit-check"
              type="checkbox"
              data-setting="export_credit"
              checked={view.export_credit}
              onchange={(event) => change({ field: "export_credit", value: event.currentTarget.checked })}
            />
          </span>
        </div>

        <div class="sl-settings-row">
          <label for="pdf-paper-select">{copy.pdfPaper}</label>
          <span class="sl-settings-control">
            <ResetButton
              {copy}
              label={copy.pdfPaper}
              changed={!isDefault(view, defaults, "pdf_paper")}
              onReset={() => reset("pdf_paper")}
            />
            <select
              id="pdf-paper-select"
              data-setting="pdf_paper"
              value={view.pdf_paper}
              onchange={(event) => change({ field: "pdf_paper", value: event.currentTarget.value })}
            >
              {#each defaults?.pdf_papers ?? [view.pdf_paper] as code (code)}
                <option value={code}>{localizedName(copy.paperNames, code)}</option>
              {/each}
            </select>
          </span>
        </div>

        {@render folderRow(folderRows[0])}
      </div>
    </section>

    <section class="sl-settings-block" aria-labelledby="settings-group-appearance">
      <h2 id="settings-group-appearance" class="sl-settings-group-label">{copy.groupAppearance}</h2>
      <div class="sl-settings-rows">
        <label class="sl-settings-row" for="language-select">
          <span>{copy.language}</span>
          <span class="sl-settings-control">
            <select
              id="language-select"
              data-setting="language"
              value={view.language === SYSTEM_LANGUAGE ? SYSTEM_LANGUAGE : localeCode}
              onchange={onLanguageChange}
              aria-label={copy.language}
            >
              <option value={SYSTEM_LANGUAGE}>{copy.languageSystem}</option>
              {#each UI_LOCALES as option (option.code)}
                <option value={option.code} lang={option.code}>{option.name}</option>
              {/each}
            </select>
          </span>
        </label>

        <label class="sl-settings-row" for="theme-select">
          <span>{copy.theme}</span>
          <span class="sl-settings-control">
            <select
              id="theme-select"
              value={asThemePreference(view.theme)}
              onchange={onThemeChange}
              aria-label={copy.theme}
            >
              <option value="system">{copy.themeSystem}</option>
              <option value="light">{copy.themeLight}</option>
              <option value="dark">{copy.themeDark}</option>
            </select>
          </span>
        </label>

        <div class="sl-settings-row">
          <label for="brand-select">{copy.brand}</label>
          <span class="sl-settings-control">
            <ResetButton
              {copy}
              label={copy.brand}
              changed={!isDefault(view, defaults, "brand")}
              onReset={() => reset("brand")}
            />
            <span class="sl-brand-swatch" style:background={presetAccent} aria-hidden="true"></span>
            <select
              id="brand-select"
              data-setting="brand"
              value={view.brand}
              onchange={(event) => change({ field: "brand", value: event.currentTarget.value })}
            >
              {#each BRANDS as preset (preset.code)}
                <option value={preset.code}>{localizedName(copy.brandNames, preset.code)}</option>
              {/each}
            </select>
          </span>
        </div>

        <div class="sl-settings-row">
          <label for="accent-color-input">{copy.accentColor}</label>
          <span class="sl-settings-control">
            <ResetButton
              {copy}
              label={copy.accentColor}
              changed={!isDefault(view, defaults, "accent_color")}
              onReset={() => reset("accent_color")}
            />
            <input
              id="accent-color-input"
              class="sl-settings-color"
              type="color"
              data-setting="accent_color"
              value={view.accent_color || presetAccent}
              onchange={(event) => change({ field: "accent_color", value: event.currentTarget.value })}
            />
          </span>
        </div>

      </div>
    </section>

    <section class="sl-settings-block" aria-labelledby="settings-group-storage">
      <h2 id="settings-group-storage" class="sl-settings-group-label">{copy.groupStorage}</h2>
      <div class="sl-settings-rows">
        {@render folderRow(folderRows[1])}
      </div>
    </section>

    <AdvancedSettings {copy} {localeCode} {view} {defaults} onChange={change} onResetAll={resetAdvanced} />
  </div>

  {#snippet folderRow(row: (typeof folderRows)[number])}
    <div class="sl-settings-row sl-settings-row-path">
      <span>{row.label}</span>
      <span class="sl-settings-control sl-settings-path">
        <span class="sl-path-text" title={row.path} data-part="folder-path">{row.shown}</span>
        <button class="sl-btn sl-btn-small" type="button" onclick={() => browseFolder(row.which)}>
          <FolderOpen class="sl-icon" />{copy.browse}
        </button>
      </span>
    </div>
    {#if row.fallback}
      <p class="sl-settings-error" role="status">{copy.pathInvalid}</p>
    {/if}
  {/snippet}

  <footer class="sl-settings-foot">
    <p class="sl-settings-footer">{copy.settingsFooter}</p>
    <button class="sl-btn sl-btn-outlined" type="button" onclick={onClose}>
      {copy.backGeneric}
    </button>
  </footer>
</section>
