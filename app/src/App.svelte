<script lang="ts">
  import { untrack } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import "./styles/soft-ledger.css";
  import "./styles/editor.css";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import FileDown from "@lucide/svelte/icons/file-down";
  import Info from "@lucide/svelte/icons/info";
  import MousePointerClick from "@lucide/svelte/icons/mouse-pointer-click";
  import PenLine from "@lucide/svelte/icons/pen-line";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import AnnotateStage from "./AnnotateStage.svelte";
  import { applyPalette, brandPalette } from "./brand";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import EmptyState from "./EmptyState.svelte";
  import CapturePill from "./CapturePill.svelte";
  import ExportSheet from "./ExportSheet.svelte";
  import Filmstrip from "./Filmstrip.svelte";
  import Library from "./Library.svelte";
  import SettingsPage from "./SettingsPage.svelte";
  import type { OverlayDto } from "./annotate";
  import type { CropRect } from "./crop";
  import {
    resolveChromeSurface,
    stepCountLabel,
    windowTitleForSurface,
    type RecorderPhase,
  } from "./chrome";
  import {
    pruneImageById,
    retainStepImageIds,
    type GuideDto,
    type ImageById,
    type OpenGuideResult,
    type StepDto,
  } from "./guide";
  import { uiLocale } from "./locales";
  import { asCommandError, reportError } from "./commandError";
  import type { RecentGuideEntry } from "./openGuide";
  import {
    DEFAULT_SETTINGS_VIEW,
    asExportFormat,
    asThemePreference,
    exportFolderKind,
    formatHotkeyLabel,
    resolveThemeSkin,
    type ExportFormat,
    type SettingsDefaults,
    type SettingsView,
  } from "./settingsUi";
  import { fillTemplate } from "./template";
  import { confirmIntent, stepTextChanged, stepTextConfirms, type Intent } from "./unsavedWork";

  type StateEvent = { state: string; steps: number };

  let recorderState = $state<RecorderPhase>("idle");
  let liveSteps = $state(0);
  let guide = $state<GuideDto | null>(null);
  let selectedId = $state<string | null>(null);
  let imageById = $state<ImageById>({});
  /** Oldest first. Together with `imageById`, bounded by `retainStepImageIds`. */
  let imageOrder = $state<string[]>([]);
  /** Filmstrip frames currently in view. */
  let visibleStepIds = $state<string[]>([]);
  let editText = $state("");
  let statusMessage = $state("");
  let statusKind = $state<"info" | "error">("info");
  let statusClearTimer: ReturnType<typeof setTimeout> | null = null;
  let selectTitleOnLoad = $state(false);
  let titleInput: HTMLInputElement | undefined = $state();
  let settingsView = $state<SettingsView>(DEFAULT_SETTINGS_VIEW);
  let settingsDefaults = $state<SettingsDefaults | null>(null);
  let systemDark = $state(false);
  let guideDirty = $state(false);
  let recentGuides = $state<RecentGuideEntry[]>([]);
  let settingsOpen = $state(false);
  let annotateMode = $state(false);
  let exportSheetOpen = $state(false);
  let exportFormat = $state<ExportFormat>("html");
  let exportOpenAfter = $state(true);
  let confirmRequest = $state<{
    message: string;
    confirmLabel: string;
    resolve: (confirmed: boolean) => void;
  } | null>(null);

  const locale = $derived(uiLocale(settingsView.resolved_language));
  const copy = $derived(locale.messages);
  const recordHotkey = $derived(formatHotkeyLabel(settingsView.hotkey, copy.keyNames));
  const pauseHotkey = $derived(formatHotkeyLabel(settingsView.pause_hotkey, copy.keyNames));
  const themeSkin = $derived(
    resolveThemeSkin(asThemePreference(settingsView.theme), systemDark),
  );
  const chromeSurface = $derived(
    resolveChromeSurface({ phase: recorderState, settingsOpen }),
  );
  const selectedStep = $derived(
    guide?.steps.find((step) => step.id === selectedId) ?? null,
  );
  const selectedImage = $derived(
    selectedId ? (imageById[selectedId] ?? null) : null,
  );
  const hasSession = $derived(
    guide !== null && guide.steps.length > 0 && recorderState === "reviewing",
  );
  const needsTitlePrompt = $derived(hasSession && guide?.title.trim() === "");
  /** 1-based position of the selected step, for the editor's step heading. */
  const selectedStepIndex = $derived(
    guide ? guide.steps.findIndex((step) => step.id === selectedId) + 1 : 0,
  );

  function clearStatus() {
    statusMessage = "";
    if (statusClearTimer !== null) {
      clearTimeout(statusClearTimer);
      statusClearTimer = null;
    }
  }

  function showStatus(message: string, kind: "info" | "error" = "info") {
    if (!message) {
      clearStatus();
      return;
    }
    statusMessage = message;
    statusKind = kind;
    if (statusClearTimer !== null) clearTimeout(statusClearTimer);
    statusClearTimer = setTimeout(() => {
      statusMessage = "";
      statusClearTimer = null;
    }, 8000);
  }

  async function refreshGuide(keepSelection = false) {
    guide = await invoke<GuideDto | null>("get_guide");
    // Argv / soft-open may fill the slot while FE still shows Idle.
    if (guide && recorderState === "idle") {
      recorderState = "reviewing";
    }
    if (!guide || guide.steps.length === 0) {
      selectedId = null;
      clearStepImages();
      return;
    }
    // A document switch (open or stop) must drop the keyed image map. Step IDs
    // restart at step-0001 across packages.
    if (!keepSelection) {
      clearStepImages();
    } else {
      imageById = pruneImageById(
        imageById,
        guide.steps.map((step) => step.id),
      );
      imageOrder = imageOrder.filter((id) => imageById[id]);
    }
    if (!keepSelection || !guide.steps.some((step) => step.id === selectedId)) {
      selectedId = guide.steps[0].id;
    }
  }

  function clearStepImages() {
    imageById = {};
    imageOrder = [];
    visibleStepIds = [];
  }

  function storeStepImage(id: string, uri: string) {
    const order = [...imageOrder.filter((entry) => entry !== id), id];
    const next = { ...imageById, [id]: uri };
    const keep = new Set(retainStepImageIds(wantedStepImages(), order));
    imageOrder = order.filter((entry) => keep.has(entry));
    imageById = Object.fromEntries(Object.entries(next).filter(([key]) => keep.has(key)));
  }

  function setVisibleSteps(ids: string[]) {
    visibleStepIds = ids;
  }

  function wantedStepImages(): string[] {
    const withImage = new Set(
      (guide?.steps ?? []).filter((step) => step.has_image).map((step) => step.id),
    );
    return [selectedId, ...visibleStepIds].filter(
      (id): id is string => typeof id === "string" && withImage.has(id),
    );
  }

  async function loadRecentGuides() {
    try {
      recentGuides = await invoke<RecentGuideEntry[]>("get_recent_guides");
    } catch {
      recentGuides = [];
    }
  }

  /** Drop menus/sheets that should not stack across Soft Ledger transitions. */
  function dismissChromeOverlays(closeSettings = false) {
    exportSheetOpen = false;
    annotateMode = false;
    if (closeSettings) settingsOpen = false;
  }

  function applyOpenedGuide(result: OpenGuideResult) {
    guide = result.guide;
    recorderState = "reviewing";
    clearStepImages();
    selectedId = result.guide.steps[0]?.id ?? null;
    guideDirty = false;
    dismissChromeOverlays();
  }

  /** Asks in the app's own dialog; a newer question cancels an open one. */
  function askConfirm(message: string, confirmLabel: string): Promise<boolean> {
    const previous = confirmRequest;
    const next = new Promise<boolean>((resolve) => {
      confirmRequest = { message, confirmLabel, resolve };
    });
    previous?.resolve(false);
    return next;
  }

  function answerConfirm(confirmed: boolean) {
    const request = confirmRequest;
    confirmRequest = null;
    request?.resolve(confirmed);
  }

  /** The one gate for intents from the app, the tray and the hotkeys. */
  async function mayProceed(intent: Intent): Promise<boolean> {
    // A global hotkey fires without blurring the text box.
    await saveText();
    const duringCapture = recorderState === "recording" || recorderState === "paused";
    return confirmIntent(
      intent,
      { dirty: guideDirty, phase: recorderState, capturedSteps: liveSteps },
      async () => {
        // The tray or a hotkey may ask while the window sits hidden in the tray.
        // During a recording the bar is too small for the dialog.
        if (duringCapture) await invoke("present_for_dialog");
        else await invoke("show_main_window");
        const confirmed = await askConfirm(copy.discardConfirm, copy.discardAction);
        // A newer question already replaced this one and owns the window.
        if (!confirmed && duringCapture && confirmRequest === null) {
          await invoke("restore_capture_pill");
        }
        return confirmed;
      },
    );
  }

  async function openPendingGuide() {
    const path = await invoke<string | null>("take_pending_open");
    if (path) await openGuideFrom(path);
  }

  async function openGuideFrom(path: string | null) {
    if (!(await mayProceed("open"))) return;
    clearStatus();
    try {
      const result = await invoke<OpenGuideResult | null>("open_guide", { path });
      if (!result) return;
      applyOpenedGuide(result);
    } catch (error) {
      showStatus(reportError(error, copy), "error");
      void loadRecentGuides();
    }
  }

  function focusTitleSelectAll() {
    const input = titleInput;
    if (!input) return;
    input.focus();
    input.select();
  }

  /** Settings defaults seed the export sheet; the sheet itself may deviate per run. */
  $effect(() => {
    exportFormat = asExportFormat(settingsView.export_format);
  });

  $effect(() => {
    exportOpenAfter = settingsView.open_after_export;
  });

  $effect(() => {
    document.title = windowTitleForSurface(chromeSurface, copy);
  });

  $effect(() => {
    document.documentElement.lang = locale.code;
  });

  $effect(() => {
    if (chromeSurface === "library") void loadRecentGuides();
  });

  $effect(() => {
    document.documentElement.dataset.theme = themeSkin;
    applyPalette(
      document.documentElement,
      brandPalette(settingsView.brand, themeSkin, settingsView.accent_color),
    );
  });

  $effect(() => {
    editText = selectedStep?.text ?? "";
  });

  $effect(() => {
    const wanted = wantedStepImages();
    const cached = untrack(() => imageById);
    const order = untrack(() => imageOrder);
    const keep = new Set(retainStepImageIds(wanted, order));
    if (order.some((id) => !keep.has(id))) {
      imageOrder = order.filter((id) => keep.has(id));
      imageById = Object.fromEntries(Object.entries(cached).filter(([id]) => keep.has(id)));
    }
    let cancelled = false;
    for (const id of wanted) {
      if (!keep.has(id) || cached[id]) continue;
      void invoke<string | null>("get_step_image", { id }).then((uri) => {
        if (cancelled || !uri) return;
        storeStepImage(id, uri);
      });
    }
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (selectTitleOnLoad && guide && recorderState === "reviewing" && titleInput) {
      selectTitleOnLoad = false;
      queueMicrotask(() => focusTitleSelectAll());
    }
  });

  $effect(() => {
    const unlisteners: Array<() => void> = [];
    listen<StateEvent>("recorder://state", (event) => {
      recorderState = event.payload.state as RecorderPhase;
      liveSteps = event.payload.steps;
      if (event.payload.state === "recording") {
        dismissChromeOverlays(true);
      }
      if (event.payload.state === "idle") {
        dismissChromeOverlays();
      }
    }).then((un) => unlisteners.push(un));
    listen("recorder://finished", () => {
      selectTitleOnLoad = true;
      guideDirty = true;
      void refreshGuide();
    }).then((un) => unlisteners.push(un));
    listen("guide://opened", () => {
      guideDirty = false;
      void refreshGuide();
    }).then((un) => unlisteners.push(un));
    listen("guide://saved", () => {
      guideDirty = false;
    }).then((un) => unlisteners.push(un));
    listen("recorder://start-requested", () => {
      void command("start");
    }).then((un) => unlisteners.push(un));
    listen<unknown>("recorder://start-failed", (event) => {
      showStatus(reportError(event.payload, copy), "error");
    }).then((un) => unlisteners.push(un));
    listen("app://quit-requested", () => {
      void quit();
    }).then((un) => unlisteners.push(un));
    listen("guide://open-requested", () => {
      void openPendingGuide();
    }).then((un) => unlisteners.push(un));
    listen<unknown>("guide://open-error", (event) => {
      showStatus(reportError(event.payload, copy), "error");
      void loadRecentGuides();
    }).then((un) => unlisteners.push(un));

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    systemDark = media.matches;
    const onScheme = (event: MediaQueryListEvent) => {
      systemDark = event.matches;
    };
    media.addEventListener("change", onScheme);

    invoke<SettingsView>("get_settings").then((view) => {
      settingsView = view;
      const taken = view.unregistered_hotkeys
        .map((slot) => (slot === "pause" ? view.pause_hotkey : view.hotkey))
        .join(", ");
      if (taken) {
        showStatus(`${copy.hotkeyRegisterError}: ${taken}`, "error");
        openSettings();
      }
    });
    invoke<SettingsDefaults>("get_settings_defaults").then((defaults) => {
      settingsDefaults = defaults;
    });
    void refreshGuide();
    void openPendingGuide();

    return () => {
      unlisteners.forEach((un) => un());
      media.removeEventListener("change", onScheme);
    };
  });

  async function command(
    name: Extract<Intent, "start" | "pause" | "resume" | "stop" | "discard">,
  ): Promise<boolean> {
    if (!(await mayProceed(name))) return false;
    clearStatus();
    await invoke("recorder_command", { name });
    return true;
  }

  async function quit() {
    if (await mayProceed("quit")) await invoke("quit_app");
  }

  /** Editor top bar, back to the library. Ask before dropping unsaved work, then clear the session. */
  async function backToLibrary() {
    if (!(await command("discard"))) return;
    dismissChromeOverlays();
    guide = null;
    selectedId = null;
    clearStepImages();
    guideDirty = false;
  }

  /** Settings covers the library or the editor. Close menus and sheets. Keep annotate mode. */
  function openSettings() {
    exportSheetOpen = false;
    settingsOpen = true;
  }

  async function saveText() {
    if (!selectedStep || !stepTextChanged(selectedStep, editText)) return;
    await invoke("set_step_text", { id: selectedStep.id, text: editText });
    guideDirty = true;
    await refreshGuide(true);
  }

  function onStepTextKey(event: KeyboardEvent) {
    if (!stepTextConfirms(event)) return;
    event.preventDefault();
    if (event.currentTarget instanceof HTMLTextAreaElement) {
      event.currentTarget.blur();
    }
  }

  async function deleteStep(id: string) {
    // Nach dem Löschen den nächsten Schritt auswählen, am Listenende den
    // vorigen. Nicht auf den ersten Schritt springen.
    const index = guide?.steps.findIndex((step) => step.id === id) ?? -1;
    const steps = await invoke<StepDto[]>("delete_step", { id });
    guide = guide ? { ...guide, steps } : null;
    guideDirty = true;
    if (steps.length === 0) {
      selectedId = null;
    } else {
      selectedId = steps[Math.min(Math.max(index, 0), steps.length - 1)].id;
    }
  }

  async function moveStep(from: number, to: number): Promise<StepDto[]> {
    if (!guide) return [];
    const id = selectedId;
    // Das Backend liefert die aktualisierte Liste direkt. Ein zweiter
    // Roundtrip entfällt.
    const steps = await invoke<StepDto[]>("move_step", { from, to });
    guide = { ...guide, steps };
    selectedId = id;
    guideDirty = true;
    return steps;
  }

  async function persistCrop(crop: CropRect | null) {
    if (!selectedStep) return;
    const steps = await invoke<StepDto[]>("set_step_crop", {
      id: selectedStep.id,
      crop,
    });
    if (guide) guide = { ...guide, steps };
    guideDirty = true;
  }

  async function persistOverlays(overlays: OverlayDto[]) {
    if (!selectedStep) return;
    const steps = await invoke<StepDto[]>("set_step_overlays", {
      id: selectedStep.id,
      overlays,
    });
    if (guide) guide = { ...guide, steps };
    guideDirty = true;
  }

  async function rememberMarkColor(color: string) {
    try {
      settingsView = await invoke<SettingsView>("change_setting", {
        change: { field: "annotation_color", value: color },
      });
    } catch (error) {
      showStatus(reportError(error, copy), "error");
    }
  }

  async function invokePathCommand(
    name: "save_guide" | "export_html" | "export_pdf",
    successPrefix: string,
  ): Promise<string | null> {
    if (!hasSession) {
      showStatus(copy.noSessionExport, "error");
      return null;
    }
    try {
      const path = await invoke<string>(name);
      showStatus(`${successPrefix}: ${path}`);
      return path;
    } catch (error) {
      showStatus(reportError(error, copy), "error");
      return null;
    }
  }

  /** Export sheet primary. Runs the invoke for that format, then closes on success. */
  function exportInvocation(format: ExportFormat): Promise<string | null> {
    switch (format) {
      case "html":
        return invokePathCommand("export_html", copy.htmlExported);
      case "pdf":
        return invokePathCommand("export_pdf", copy.pdfExported);
      case "steps":
        return invokePathCommand("save_guide", copy.saved);
      default: {
        const _exhaustive: never = format;
        return _exhaustive;
      }
    }
  }

  async function runExport() {
    const path = await exportInvocation(exportFormat);
    if (!path) return;
    exportSheetOpen = false;
    if (exportOpenAfter) {
      try {
        await invoke("reveal_path", { path });
      } catch {
        /* reveal is best-effort */
      }
    }
  }

  /** Changes whichever folder the selected format writes to. */
  async function changeExportFolder() {
    const toExportFolder = exportFolderKind(exportFormat) === "export";
    const command = toExportFolder ? "set_default_export_folder" : "set_guides_folder";
    try {
      const picked = await invoke<string | null>("pick_folder", {
        title: toExportFolder ? copy.defaultFolder : copy.guidesFolder,
      });
      if (!picked) return;
      settingsView = await invoke<SettingsView>(command, { path: picked });
    } catch (error) {
      const invalid = asCommandError(error).code === "invalid_folder";
      showStatus(invalid ? copy.pathInvalid : reportError(error, copy), "error");
    }
  }

  async function updateTitle() {
    if (!guide) return;
    await invoke("set_guide_title", { title: guide.title });
    guideDirty = true;
  }
</script>

<main class="soft-ledger" data-surface={chromeSurface}>
  <p class="sl-sr-only" role="status" aria-live="polite">
    {copy.stateLabels[recorderState]}
  </p>

  {#if statusMessage && chromeSurface !== "capture"}
    <p
      class="sl-status"
      data-kind={statusKind}
      role={statusKind === "error" ? "alert" : "status"}
      aria-live={statusKind === "error" ? "assertive" : "polite"}
    >
      {#if statusKind === "error"}<CircleAlert class="sl-icon" />{:else}<Info class="sl-icon" />{/if}
      <span>{statusMessage}</span>
    </p>
  {/if}

  {#if chromeSurface === "library"}
    <Library
      {copy}
      localeCode={locale.code}
      {recentGuides}
      captureHotkey={recordHotkey}
      onStart={() => command("start")}
      onOpen={openGuideFrom}
      onSettings={openSettings}
    />
  {:else if chromeSurface === "capture"}
    <CapturePill
      {copy}
      {recorderState}
      stepLabel={stepCountLabel(copy, liveSteps)}
      recordHotkey={{ chord: settingsView.hotkey, label: recordHotkey }}
      pauseHotkey={{ chord: settingsView.pause_hotkey, label: pauseHotkey }}
      onCommand={command}
    />
  {:else if chromeSurface === "editor"}
    {#if guide && guide.steps.length > 0}
      <section class="sl-editor">
        <div class="sl-editor-bar">
          <button class="sl-back" type="button" data-action="back-library" onclick={backToLibrary}>
            <ArrowLeft class="sl-icon" />{copy.backLibrary}
          </button>
          <input
            id="guide-title"
            class="sl-title-input"
            bind:this={titleInput}
            bind:value={guide.title}
            onchange={updateTitle}
            placeholder={copy.titlePlaceholder}
            aria-label={copy.titlePlaceholder}
          />
          {#if needsTitlePrompt}
            <p class="sl-title-prompt" role="status">{copy.titlePrompt}</p>
          {/if}
          <button
            class="sl-annotate-toggle"
            class:active={annotateMode}
            type="button"
            aria-pressed={annotateMode}
            data-action="toggle-annotate"
            onclick={() => (annotateMode = !annotateMode)}
          >
            <PenLine class="sl-icon" />
            {copy.annotateMode}
          </button>
          <button
            class="sl-btn sl-btn-primary sl-editor-export"
            type="button"
            data-action="open-export"
            onclick={() => (exportSheetOpen = true)}
          >
            <FileDown class="sl-icon" />{copy.exportPrimary}
          </button>
        </div>

        <Filmstrip
          {copy}
          steps={guide.steps}
          bind:selectedId
          {imageById}
          countLabel={stepCountLabel(copy, guide.steps.length)}
          onBeforeReorder={saveText}
          onMove={moveStep}
          onVisibleSteps={setVisibleSteps}
        />

        <div class="sl-editor-body">
          <div class="sl-stage-col">
            {#if selectedStep}
              <div class="sl-step-head">
                <p class="sl-step-label">
                  {fillTemplate(copy.stepHeading, {
                    n: selectedStepIndex,
                    text: selectedStep.text || selectedStep.action,
                  })}
                </p>
                <div class="sl-detail-tools">
                  <span class="sl-action-chip">{selectedStep.action}</span>
                  <button
                    class="sl-btn-danger-text"
                    type="button"
                    onclick={() => deleteStep(selectedStep!.id)}
                  >
                    <Trash2 class="sl-icon" />{copy.deleteStep}
                  </button>
                </div>
              </div>
              <textarea
                class="sl-step-text"
                rows="2"
                bind:value={editText}
                onblur={saveText}
                onkeydown={onStepTextKey}
                aria-label={copy.stepTextLabel}
                aria-keyshortcuts="Enter"
                aria-describedby="step-text-hint"
              ></textarea>
              <p class="sl-hint" id="step-text-hint">{copy.stepTextHint}</p>
              {#if selectedStep.has_override}
                <p class="sl-hint">{copy.overrideHint}</p>
              {/if}
              {#if annotateMode}
                <p class="sl-annotate-micro">{copy.annotateActiveMicro}</p>
              {/if}
              {#if selectedImage}
                <AnnotateStage
                  {copy}
                  step={selectedStep}
                  image={selectedImage}
                  defaultColor={settingsView.annotation_color}
                  strokeWidth={settingsView.annotation_stroke}
                  bind:annotateMode
                  onPersist={persistOverlays}
                  onConfirm={askConfirm}
                  onRememberColor={rememberMarkColor}
                  onCrop={persistCrop}
                />
              {:else if selectedStep.has_image}
                <p class="sl-hint">{copy.imageLoading}</p>
              {:else if selectedStep.action === "text_input"}
                <p class="sl-hint">{copy.noScreenshotTyping}</p>
              {:else}
                <p class="sl-hint">{copy.noScreenshot}</p>
              {/if}
            {:else}
              <p class="sl-hint">{copy.noStepSelected}</p>
            {/if}
          </div>
        </div>
      </section>
    {:else}
      <section class="sl-empty">
        <button class="sl-back" type="button" data-action="back-library" onclick={backToLibrary}>
          <ArrowLeft class="sl-icon" />{copy.backLibrary}
        </button>
        <EmptyState icon={MousePointerClick} title={copy.emptyEditorTitle} hint={copy.emptyEditorHint}>
          <button class="sl-btn sl-btn-primary" type="button" onclick={() => command("start")}>
            {copy.newCapture}
          </button>
        </EmptyState>
      </section>
    {/if}
  {:else if chromeSurface === "settings"}
    <SettingsPage
      {copy}
      localeCode={locale.code}
      bind:view={settingsView}
      defaults={settingsDefaults}
      onClose={() => (settingsOpen = false)}
      onError={(message) => showStatus(message, "error")}
    />
  {/if}

  {#if exportSheetOpen}
    <ExportSheet
      {copy}
      bind:format={exportFormat}
      bind:openAfter={exportOpenAfter}
      targetFolder={exportFolderKind(exportFormat) === "export"
        ? settingsView.export_folder_display
        : settingsView.guides_folder_display}
      targetSynced={exportFolderKind(exportFormat) === "export"
        ? settingsView.export_folder_synced
        : settingsView.guides_folder_synced}
      onChangeFolder={changeExportFolder}
      onRun={runExport}
      onClose={() => (exportSheetOpen = false)}
    />
  {/if}

  {#if confirmRequest}
    <ConfirmDialog
      message={confirmRequest.message}
      confirmLabel={confirmRequest.confirmLabel}
      cancelLabel={copy.confirmCancel}
      onAnswer={answerConfirm}
    />
  {/if}
</main>
