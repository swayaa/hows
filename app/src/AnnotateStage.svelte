<script lang="ts">
  import "./styles/annotate-stage.css";
  import {
    COLOR_PRESETS,
    MARKS,
    arrowHeadPoints,
    circleFromCornerDrag,
    circleRadii,
    hitTestHandle,
    hitTestOverlay,
    moveOverlay,
    normalizeBox,
    overlayPath,
    paintOrder,
    penStroke,
    recolorOverlay,
    resizeOverlay,
    selectionHandleList,
    type AnnotateTool,
    type OverlayDto,
    type ResizeHandle,
    type ShotSize,
  } from "./annotate";
  import type { Messages } from "./locales";
  import type { Component } from "svelte";
  import {
    hitCropHandle,
    moveCrop,
    resizeCrop,
    type CropHandle,
    type CropRect,
  } from "./crop";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Crop from "@lucide/svelte/icons/crop";
  import Circle from "@lucide/svelte/icons/circle";
  import Eraser from "@lucide/svelte/icons/eraser";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Highlighter from "@lucide/svelte/icons/highlighter";
  import MoveUpRight from "@lucide/svelte/icons/move-up-right";
  import PenLine from "@lucide/svelte/icons/pen-line";
  import Square from "@lucide/svelte/icons/square";
  import Type from "@lucide/svelte/icons/type";
  import Undo2 from "@lucide/svelte/icons/undo-2";
  import type { StepDto } from "./guide";

  let {
    copy,
    step,
    image,
    defaultColor,
    strokeWidth,
    annotateMode = $bindable(),
    onPersist,
    onConfirm,
    onRememberColor,
    onCrop,
  }: {
    copy: Messages;
    /** The selected step; its overlays are the committed marks. */
    step: StepDto;
    /** Loaded `data:` URI of the step screenshot. */
    image: string;
    /** Mark color from the settings, used until a preset is picked. */
    defaultColor: string;
    /** Line width for shapes from the settings; the pen scales along. */
    strokeWidth: number;
    annotateMode: boolean;
    /** Replace the step's overlays in the backend; resolves after the guide refreshed. */
    onPersist: (overlays: OverlayDto[]) => Promise<void>;
    /** Asks before a destructive action; resolves true when the user agrees. */
    onConfirm: (message: string, confirmLabel: string) => Promise<boolean>;
    /** Stores the chosen mark color for the next marks. */
    onRememberColor: (color: string) => void;
    /** Stores the visible window. `null` shows the whole screenshot again. */
    onCrop: (crop: CropRect | null) => Promise<void>;
  } = $props();

  type AnnotateGesture =
    | { kind: "idle" }
    | { kind: "drawing"; originX: number; originY: number }
    | {
        kind: "moving";
        id: string;
        start: { x: number; y: number };
        origin: OverlayDto;
      }
    | { kind: "resizing"; id: string; handle: ResizeHandle; origin: OverlayDto };

  const ANNOTATE_TOOLS: AnnotateTool[] = [
    "rect",
    "arrow",
    "circle",
    "pen",
    "highlight",
    "blur",
    "text",
  ];

  const TOOL_ICONS: Record<AnnotateTool, Component<{ class?: string }>> = {
    rect: Square,
    arrow: MoveUpRight,
    circle: Circle,
    pen: PenLine,
    highlight: Highlighter,
    blur: EyeOff,
    text: Type,
  };

  let annotateTool = $state<AnnotateTool>("rect");
  let pickedColor = $state<string | null>(null);
  const strokeColor = $derived(pickedColor ?? defaultColor);
  const penWidth = $derived(penStroke(strokeWidth));
  let colorMenuOpen = $state(false);
  let selectedOverlayId = $state<string | null>(null);
  let draftOverlay = $state<OverlayDto | null>(null);
  let annotateGesture = $state<AnnotateGesture>({ kind: "idle" });
  let shotEl: HTMLDivElement | undefined = $state();
  let natural = $state({ width: 0, height: 0 });
  let cropping = $state(false);
  let cropDraft = $state<CropRect | null>(null);
  let cropDrag = $state<{
    handle: CropHandle;
    origin: CropRect;
    startX: number;
    startY: number;
  } | null>(null);
  let shotSize = $state<ShotSize>({ width: 1, height: 1 });
  let textDraft = $state<{ id: string; x: number; y: number; value: string } | null>(
    null,
  );
  let textInputEl: HTMLInputElement | undefined = $state();

  const stepOverlays = $derived(step.overlays);
  const liveOverlays = $derived.by(() => {
    if (!draftOverlay) return stepOverlays;
    const without = stepOverlays.filter((m) => m.id !== draftOverlay!.id);
    return [...without, draftOverlay];
  });
  const canvasOverlays = $derived(paintOrder(liveOverlays));
  const selectedOverlay = $derived(
    liveOverlays.find((m) => m.id === selectedOverlayId) ?? null,
  );
  const selectionHandles = $derived(
    selectedOverlay ? selectionHandleList(selectedOverlay, shotSize) : [],
  );
  /** Pill and pointer tools are live only in annotate mode (Editor surface). */
  const showAnnotate = $derived(annotateMode);
  const appliedCrop = $derived(step.crop);
  const clipping = $derived(appliedCrop != null && !cropping);

  $effect(() => {
    void step.id;
    clearAnnotateDraftOnStepChange();
  });

  $effect(() => {
    if (textDraft && textInputEl) {
      queueMicrotask(() => textInputEl?.focus());
    }
  });

  $effect(() => {
    if (!showAnnotate) return;
    const onKey = (event: KeyboardEvent) => onShotKeydown(event);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  $effect(() => {
    if (!cropping) return;
    const onKey = (event: KeyboardEvent) => onCropKey(event);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  $effect(() => {
    if (annotateMode) return;
    // Verlassen des Annotate-Modus räumt Auswahl und Entwurf auf. Sonst bleibt
    // eine Markierung ausgewählt, die im Anzeige-Modus nicht sichtbar ist.
    selectedOverlayId = null;
    colorMenuOpen = false;
    textDraft = null;
    cropping = false;
    cropDraft = null;
    cropDrag = null;
  });

  function clearAnnotateDraftOnStepChange() {
    draftOverlay = null;
    cropping = false;
    cropDraft = null;
    cropDrag = null;
    annotateGesture = { kind: "idle" };
    textDraft = null;
    selectedOverlayId = null;
    colorMenuOpen = false;
  }

  function syncShotSize() {
    if (!shotEl) return;
    const rect = shotEl.getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0) {
      shotSize = { width: rect.width, height: rect.height };
    }
  }

  function toolColor(): string {
    if (annotateTool === "highlight") {
      return strokeColor === defaultColor ? MARKS.highlight_color : strokeColor;
    }
    return strokeColor;
  }

  function applyColorPreset(color: string) {
    pickedColor = color;
    colorMenuOpen = false;
    onRememberColor(color);
    const selected = selectedOverlay;
    if (!selected || selected.type === "blur") return;
    const next = liveOverlays.map((m) =>
      m.id === selected.id ? recolorOverlay(m, color) : m,
    );
    void persistOverlays(next);
  }

  function newOverlayId(): string {
    return `o${Date.now().toString(36)}${Math.random().toString(36).slice(2, 6)}`;
  }

  function normFromEvent(event: PointerEvent): { x: number; y: number } | null {
    const el = shotEl;
    if (!el) return null;
    const rect = el.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return null;
    shotSize = { width: rect.width, height: rect.height };
    const x = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width));
    const y = Math.min(1, Math.max(0, (event.clientY - rect.top) / rect.height));
    return { x, y };
  }

  async function persistOverlays(next: OverlayDto[]) {
    await onPersist(next);
  }

  function beginAnnotateGesture(
    event: PointerEvent,
    gesture: Exclude<AnnotateGesture, { kind: "idle" }>,
    draft: OverlayDto | null,
  ) {
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    annotateGesture = gesture;
    draftOverlay = draft;
  }

  /** Empty draft for the active tool at `point`; null for the text tool (click-to-place). */
  function newDraft(point: { x: number; y: number }): OverlayDto | null {
    const id = newOverlayId();
    const color = toolColor();
    const box = { x: point.x, y: point.y, w: 0, h: 0 };
    switch (annotateTool) {
      case "rect":
        return { type: "rect", id, color, stroke: strokeWidth, ...box };
      case "highlight":
        return { type: "highlight", id, color, opacity: MARKS.highlight_opacity, ...box };
      case "blur":
        return { type: "blur", id, ...box };
      case "arrow":
        return {
          type: "arrow",
          id,
          color,
          stroke: strokeWidth,
          x1: point.x,
          y1: point.y,
          x2: point.x,
          y2: point.y,
        };
      case "circle":
        return { type: "circle", id, color, stroke: strokeWidth, cx: point.x, cy: point.y, r: 0 };
      case "pen":
        return { type: "pen", id, color, stroke: penWidth, points: [[point.x, point.y]] };
      case "text":
        return null;
      default: {
        const _exhaustive: never = annotateTool;
        return _exhaustive;
      }
    }
  }

  function startDraw(point: { x: number; y: number }) {
    selectedOverlayId = null;
    textDraft = null;
    const draft = newDraft(point);
    if (!draft) return;
    annotateGesture = { kind: "drawing", originX: point.x, originY: point.y };
    draftOverlay = draft;
  }

  function beginCrop() {
    cropping = true;
    cropDraft = step.crop ?? { x: 0.08, y: 0.08, w: 0.84, h: 0.84 };
    cropDrag = null;
    draftOverlay = null;
    textDraft = null;
    selectedOverlayId = null;
    annotateGesture = { kind: "idle" };
  }

  function cancelCrop() {
    cropping = false;
    cropDraft = null;
    cropDrag = null;
  }

  async function applyCrop() {
    const draft = cropDraft;
    if (!draft) return;
    cancelCrop();
    await onCrop(draft);
  }

  async function resetCrop() {
    cancelCrop();
    await onCrop(null);
  }

  function onCropPointerDown(event: PointerEvent) {
    if (event.button !== 0 || !cropDraft) return;
    syncShotSize();
    const point = normFromEvent(event);
    if (!point) return;
    const handle = hitCropHandle(cropDraft, point, shotSize);
    if (!handle) return;
    event.preventDefault();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    cropDrag = { handle, origin: cropDraft, startX: point.x, startY: point.y };
  }

  function onCropKey(event: KeyboardEvent) {
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      cancelCrop();
    } else if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void applyCrop();
    }
  }

  function onAnnotatePointerDown(event: PointerEvent) {
    if (cropping) {
      onCropPointerDown(event);
      return;
    }
    if (!showAnnotate || event.button !== 0) return;
    const target = event.target as HTMLElement | null;
    if (target?.closest?.(".text-draft") || target?.closest?.(".annotate-pill")) {
      return;
    }
    const point = normFromEvent(event);
    if (!point) return;
    event.preventDefault();
    syncShotSize();

    if (annotateTool === "text") {
      selectedOverlayId = null;
      textDraft = {
        id: newOverlayId(),
        x: point.x,
        y: point.y,
        value: "",
      };
      return;
    }

    // Handles / selected body first (implicit select + transform).
    if (selectedOverlay) {
      const handle = hitTestHandle(selectedOverlay, point, shotSize);
      if (handle) {
        beginAnnotateGesture(
          event,
          { kind: "resizing", id: selectedOverlay.id, handle, origin: selectedOverlay },
          selectedOverlay,
        );
        return;
      }
      if (hitTestOverlay([selectedOverlay], point, shotSize)) {
        beginAnnotateGesture(
          event,
          { kind: "moving", id: selectedOverlay.id, start: point, origin: selectedOverlay },
          selectedOverlay,
        );
        return;
      }
    }

    const hit = hitTestOverlay(stepOverlays, point, shotSize);
    if (hit) {
      selectedOverlayId = hit.id;
      textDraft = null;
      beginAnnotateGesture(
        event,
        { kind: "moving", id: hit.id, start: point, origin: hit },
        hit,
      );
      return;
    }

    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    startDraw(point);
  }

  function updateDrawingDraft(
    draft: OverlayDto,
    gesture: Extract<AnnotateGesture, { kind: "drawing" }>,
    point: { x: number; y: number },
  ): OverlayDto {
    switch (draft.type) {
      case "rect":
      case "highlight":
      case "blur":
        return {
          ...draft,
          w: point.x - gesture.originX,
          h: point.y - gesture.originY,
        };
      case "arrow":
        return { ...draft, x2: point.x, y2: point.y };
      case "circle":
        return {
          ...draft,
          ...circleFromCornerDrag(
            gesture.originX,
            gesture.originY,
            point.x,
            point.y,
            shotSize,
          ),
        };
      case "pen": {
        const points = draft.points;
        const last = points[points.length - 1];
        if (last && Math.hypot(point.x - last[0], point.y - last[1]) <= 0.002) {
          return draft;
        }
        return { ...draft, points: [...points, [point.x, point.y]] };
      }
      case "text":
        return draft;
      default: {
        const _exhaustive: never = draft;
        return _exhaustive;
      }
    }
  }

  function onAnnotatePointerMove(event: PointerEvent) {
    if (cropDrag && cropDraft) {
      const point = normFromEvent(event);
      if (!point) return;
      cropDraft =
        cropDrag.handle === "move"
          ? moveCrop(cropDrag.origin, point.x - cropDrag.startX, point.y - cropDrag.startY)
          : resizeCrop(cropDrag.origin, cropDrag.handle, point.x, point.y);
      return;
    }
    const gesture = annotateGesture;
    if (gesture.kind === "idle") return;
    const point = normFromEvent(event);
    if (!point) return;

    if (gesture.kind === "moving") {
      draftOverlay = moveOverlay(
        gesture.origin,
        point.x - gesture.start.x,
        point.y - gesture.start.y,
      );
      return;
    }
    if (gesture.kind === "resizing") {
      draftOverlay = resizeOverlay(gesture.origin, gesture.handle, point, shotSize);
      return;
    }
    if (!draftOverlay) return;
    draftOverlay = updateDrawingDraft(draftOverlay, gesture, point);
  }

  /** Drop tiny / incomplete draws; normalize boxes. Returns null if discarded. */
  function finalizeDrawnOverlay(mark: OverlayDto): OverlayDto | null {
    switch (mark.type) {
      case "rect":
      case "highlight":
      case "blur": {
        const box = normalizeBox(mark.x, mark.y, mark.w, mark.h);
        if (box.w < 0.005 && box.h < 0.005) return null;
        return { ...mark, ...box };
      }
      case "arrow":
        if (Math.hypot(mark.x2 - mark.x1, mark.y2 - mark.y1) < 0.008) return null;
        return mark;
      case "circle":
        if (mark.r < 0.004) return null;
        return mark;
      case "pen":
        if (mark.points.length < 2) return null;
        return mark;
      case "text":
        return null;
      default: {
        const _exhaustive: never = mark;
        return _exhaustive;
      }
    }
  }

  async function onAnnotatePointerUp(event: PointerEvent) {
    if (cropDrag) {
      cropDrag = null;
      try {
        (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
      } catch {
        // Capture bereits freigegeben.
      }
      return;
    }
    const gesture = annotateGesture;
    if (gesture.kind === "idle") return;
    annotateGesture = { kind: "idle" };
    try {
      (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    } catch {
      // Capture bereits freigegeben.
    }

    if (gesture.kind === "moving" || gesture.kind === "resizing") {
      const finished = draftOverlay;
      draftOverlay = null;
      if (!finished) return;
      const next = stepOverlays.map((m) => (m.id === finished.id ? finished : m));
      selectedOverlayId = finished.id;
      await persistOverlays(next);
      return;
    }

    const finished = draftOverlay ? finalizeDrawnOverlay(draftOverlay) : null;
    draftOverlay = null;
    if (!finished) return;
    selectedOverlayId = finished.id;
    await persistOverlays([...stepOverlays, finished]);
  }

  async function commitTextDraft() {
    const draft = textDraft;
    textDraft = null;
    if (!draft) return;
    const value = draft.value.trim();
    if (!value) return;
    const overlay: OverlayDto = {
      type: "text",
      id: draft.id,
      color: toolColor(),
      size: MARKS.text_size,
      x: draft.x,
      y: draft.y,
      text: value,
    };
    selectedOverlayId = overlay.id;
    await persistOverlays([...stepOverlays, overlay]);
  }

  function cancelTextDraft() {
    textDraft = null;
  }

  function onTextDraftKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      void commitTextDraft();
    } else if (event.key === "Escape") {
      event.preventDefault();
      cancelTextDraft();
    }
  }

  async function undoOverlay() {
    if (stepOverlays.length === 0) return;
    const removed = stepOverlays[stepOverlays.length - 1];
    if (selectedOverlayId === removed?.id) selectedOverlayId = null;
    await persistOverlays(stepOverlays.slice(0, -1));
  }

  async function clearOverlays() {
    if (stepOverlays.length === 0) return;
    if (
      stepOverlays.length >= 3 &&
      !(await onConfirm(copy.annotateClearConfirm, copy.annotateClearAction))
    ) {
      return;
    }
    selectedOverlayId = null;
    await persistOverlays([]);
  }

  async function deleteSelectedOverlay() {
    if (!selectedOverlayId) return;
    const next = stepOverlays.filter((m) => m.id !== selectedOverlayId);
    selectedOverlayId = null;
    await persistOverlays(next);
  }

  function onShotKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      selectedOverlayId = null;
      colorMenuOpen = false;
      return;
    }
    if (
      (event.key === "Backspace" || event.key === "Delete") &&
      selectedOverlayId &&
      !(event.target instanceof HTMLInputElement) &&
      !(event.target instanceof HTMLTextAreaElement)
    ) {
      event.preventDefault();
      void deleteSelectedOverlay();
    }
  }

</script>

<div class="sl-stage-wrap">
  <div
    class="crop-frame"
    class:clipping
    style:aspect-ratio={clipping && natural.width > 0 && appliedCrop
      ? `${appliedCrop.w * natural.width} / ${appliedCrop.h * natural.height}`
      : undefined}
  >
  <div
    class="shot"
    class:drawing={showAnnotate && !cropping}
    class:cropping
    style:width={clipping && appliedCrop ? `${100 / appliedCrop.w}%` : undefined}
    style:transform={clipping && appliedCrop
      ? `translate(${-appliedCrop.x * 100}%, ${-appliedCrop.y * 100}%)`
      : undefined}
    role="application"
    aria-label={copy.annotateToolbar}
    bind:this={shotEl}
    onpointerdown={onAnnotatePointerDown}
    onpointermove={onAnnotatePointerMove}
    onpointerup={onAnnotatePointerUp}
    onpointercancel={onAnnotatePointerUp}
  >
    <img
      src={image}
      alt={copy.screenshotAlt}
      draggable="false"
      onload={(event) => {
        const img = event.currentTarget;
        if (!(img instanceof HTMLImageElement)) return;
        natural = { width: img.naturalWidth, height: img.naturalHeight };
        syncShotSize();
      }}
    />
    {#each liveOverlays as mark (mark.id)}
      {#if mark.type === "blur"}
        {@const box = normalizeBox(mark.x, mark.y, mark.w, mark.h)}
        <div
          class="blur-mask"
          class:selected={mark.id === selectedOverlayId}
          style:left="{box.x * 100}%"
          style:top="{box.y * 100}%"
          style:width="{box.w * 100}%"
          style:height="{box.h * 100}%"
          style:border-color={MARKS.blur_outline}
          aria-hidden="true"
        ></div>
      {/if}
    {/each}
    <svg
      class="overlay-layer"
      viewBox="0 0 1 1"
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      {#each canvasOverlays as mark (mark.id)}
        {#if mark.type === "highlight"}
          {@const box = normalizeBox(mark.x, mark.y, mark.w, mark.h)}
          <rect
            x={box.x}
            y={box.y}
            width={box.w}
            height={box.h}
            fill={mark.color}
            fill-opacity={mark.opacity}
            stroke="none"
          />
        {:else if mark.type === "rect"}
          {@const box = normalizeBox(mark.x, mark.y, mark.w, mark.h)}
          <rect
            x={box.x}
            y={box.y}
            width={box.w}
            height={box.h}
            fill="none"
            stroke={mark.color}
            stroke-width={mark.stroke}
            vector-effect="non-scaling-stroke"
          />
        {:else if mark.type === "arrow"}
          <line
            x1={mark.x1}
            y1={mark.y1}
            x2={mark.x2}
            y2={mark.y2}
            stroke={mark.color}
            stroke-width={mark.stroke}
            stroke-linecap="round"
            vector-effect="non-scaling-stroke"
          />
          <polygon
            points={arrowHeadPoints(
              mark.x1,
              mark.y1,
              mark.x2,
              mark.y2,
              shotSize,
              mark.stroke,
            )}
            fill={mark.color}
          />
        {:else if mark.type === "circle"}
          {@const radii = circleRadii(mark.r, shotSize)}
          <ellipse
            cx={mark.cx}
            cy={mark.cy}
            rx={radii.rx}
            ry={radii.ry}
            fill="none"
            stroke={mark.color}
            stroke-width={mark.stroke}
            vector-effect="non-scaling-stroke"
          />
        {:else if mark.type === "pen"}
          <path
            d={overlayPath(mark.points)}
            fill="none"
            stroke={mark.color}
            stroke-width={mark.stroke}
            stroke-linecap="round"
            stroke-linejoin="round"
            vector-effect="non-scaling-stroke"
          />
        {/if}
      {/each}
    </svg>
    {#each liveOverlays as mark (mark.id)}
      {#if mark.type === "text"}
        <span
          class="overlay-text"
          class:selected={mark.id === selectedOverlayId}
          style:left="{mark.x * 100}%"
          style:top="{mark.y * 100}%"
          style:color={mark.color}
          style:font-size="{mark.size}px"
        >{mark.text}</span>
      {/if}
    {/each}
    {#if selectedOverlay && selectionHandles.length > 0}
      <div class="selection-layer" aria-label={copy.annotateSelected}>
        {#each selectionHandles as handle (handle.key)}
          <span
            class="sel-handle"
            style:left="{handle.x * 100}%"
            style:top="{handle.y * 100}%"
            style:cursor={handle.cursor}
          ></span>
        {/each}
      </div>
    {/if}
    {#if cropping && cropDraft}
      <div
        class="crop-shade"
        style:left="{cropDraft.x * 100}%"
        style:top="{cropDraft.y * 100}%"
        style:width="{cropDraft.w * 100}%"
        style:height="{cropDraft.h * 100}%"
      >
        <span class="crop-handle" style:left="0%" style:top="0%"></span>
        <span class="crop-handle" style:left="100%" style:top="0%"></span>
        <span class="crop-handle" style:left="0%" style:top="100%"></span>
        <span class="crop-handle" style:left="100%" style:top="100%"></span>
      </div>
    {/if}
    {#if textDraft}
      <input
        class="text-draft"
        bind:this={textInputEl}
        bind:value={textDraft.value}
        style:left="{textDraft.x * 100}%"
        style:top="{textDraft.y * 100}%"
        placeholder={copy.annotateTextPlaceholder}
        aria-label={copy.tools.text}
        onkeydown={onTextDraftKeydown}
        onblur={() => void commitTextDraft()}
      />
    {/if}
  </div>
  </div>
  {#if showAnnotate}
    <div
      class="annotate-pill"
      role="toolbar"
      aria-label={copy.annotateToolbar}
    >
      {#if cropping}
        <p class="crop-pill-hint">{copy.cropHint}</p>
        <button class="pill-done" type="button" data-action="apply-crop" onclick={() => void applyCrop()}>
          {copy.cropApply}
        </button>
        <button type="button" data-action="cancel-crop" onclick={cancelCrop}>
          {copy.cropCancel}
        </button>
        {#if step.crop}
          <button type="button" data-action="reset-crop" onclick={() => void resetCrop()}>
            {copy.cropReset}
          </button>
        {/if}
      {:else}
      {#each ANNOTATE_TOOLS as tool}
        {@const ToolIcon = TOOL_ICONS[tool]}
        <button
          type="button"
          class="pill-icon"
          class:active={annotateTool === tool}
          aria-pressed={annotateTool === tool}
          aria-label={copy.tools[tool]}
          title={copy.tools[tool]}
          data-tool={tool}
          onclick={() => (annotateTool = tool)}
        ><ToolIcon class="sl-icon" /></button>
      {/each}
      <span class="sl-pill-sep" aria-hidden="true"></span>
      <button
        type="button"
        data-action="start-crop"
        title={copy.cropHint}
        aria-label={copy.crop}
        onclick={beginCrop}
      ><Crop class="sl-icon" />{copy.crop}</button>
      <div class="color-menu">
        <button
          type="button"
          class="color-toggle"
          aria-expanded={colorMenuOpen}
          aria-haspopup="true"
          title={copy.annotateColor}
          onclick={() => (colorMenuOpen = !colorMenuOpen)}
        >
          <span
            class="color-swatch"
            style:background={strokeColor}
            aria-hidden="true"
          ></span>
          {copy.annotateColor}
          <ChevronDown class="sl-icon" />
        </button>
        {#if colorMenuOpen}
          <div class="color-presets" role="menu" aria-label={copy.annotateColor}>
            <input
              class="color-custom"
              type="color"
              aria-label={copy.annotationColor}
              value={strokeColor}
              onchange={(event) => applyColorPreset(event.currentTarget.value)}
            />
            {#each COLOR_PRESETS as preset}
              <button
                type="button"
                role="menuitem"
                class="color-swatch large"
                class:active={strokeColor === preset}
                style:background={preset}
                title={preset}
                aria-label={preset}
                onclick={() => applyColorPreset(preset)}
              ></button>
            {/each}
          </div>
        {/if}
      </div>
      <span class="sl-pill-sep" aria-hidden="true"></span>
      <button
        type="button"
        class="pill-icon"
        onclick={undoOverlay}
        disabled={stepOverlays.length === 0}
        title={copy.annotateUndo}
        aria-label={copy.annotateUndo}
      ><Undo2 class="sl-icon" /></button>
      <button
        type="button"
        class="pill-icon"
        onclick={clearOverlays}
        disabled={stepOverlays.length === 0}
        title={copy.annotateClear}
        aria-label={copy.annotateClear}
      ><Eraser class="sl-icon" /></button>
      <span class="sl-pill-sep" aria-hidden="true"></span>
      <button
        type="button"
        class="pill-done"
        onclick={() => (annotateMode = false)}
      ><Check class="sl-icon" />{copy.annotateDone}</button>
      {/if}
    </div>
  {/if}
</div>
