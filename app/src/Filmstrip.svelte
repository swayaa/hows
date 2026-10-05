<script lang="ts">
  import "./styles/filmstrip.css";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import { fillCount } from "./chrome";
  import type { Messages } from "./locales";
  import type { ImageById, StepDto } from "./guide";
  import {
    gapFromClientX,
    moveTargetForGap,
    slotAfterKeyboard,
    type ColRect,
    type DragSession,
  } from "./stepReorder";

  let {
    copy,
    steps,
    selectedId = $bindable(),
    imageById,
    countLabel,
    onBeforeReorder,
    onMove,
    onVisibleSteps,
  }: {
    copy: Messages;
    steps: StepDto[];
    selectedId: string | null;
    imageById: ImageById;
    /** "3 Schritte" for the header meta. */
    countLabel: string;
    /** Flush pending step-text edits before indices are re-read. */
    onBeforeReorder: () => Promise<void>;
    /** Persist the reorder; resolves with the reordered steps. */
    onMove: (from: number, to: number) => Promise<StepDto[]>;
    /** Step ids whose frames are in view, so the editor can load those screenshots. */
    onVisibleSteps: (ids: string[]) => void;
  } = $props();

  let filmstripEl: HTMLOListElement | undefined = $state();
  let drag = $state<DragSession>({ kind: "idle" });
  let reorderBusy = $state(false);
  let reorderAnnounce = $state("");

  const dragActive = $derived(drag.kind === "dragging" ? drag : null);

  $effect(() => {
    const root = filmstripEl;
    const listed = steps.map((step) => step.id).join("\0");
    if (!root || listed.length === 0) {
      onVisibleSteps([]);
      return;
    }
    const visible = new Set<string>();
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const id = (entry.target as HTMLElement).dataset.stepId;
          if (!id) continue;
          if (entry.isIntersecting) visible.add(id);
          else visible.delete(id);
        }
        onVisibleSteps([...visible]);
      },
      { root, rootMargin: "160px" },
    );
    for (const node of root.querySelectorAll<HTMLElement>("[data-step-id]")) {
      observer.observe(node);
    }
    return () => {
      observer.disconnect();
      onVisibleSteps([]);
    };
  });

  function focusSelectedStepButton() {
    if (!filmstripEl || !selectedId) return;
    const button = filmstripEl.querySelector<HTMLButtonElement>(
      `button.sl-film-frame[data-step-id="${CSS.escape(selectedId)}"]`,
    );
    if (!button) return;
    button.focus();
    button.scrollIntoView({ block: "nearest", inline: "center" });
  }

  function selectStepAt(index: number) {
    if (steps.length === 0) return;
    const clamped = Math.min(Math.max(index, 0), steps.length - 1);
    selectedId = steps[clamped].id;
    queueMicrotask(() => focusSelectedStepButton());
  }

  function moveSelection(delta: number) {
    if (steps.length === 0) return;
    const current = steps.findIndex((step) => step.id === selectedId);
    selectStepAt(current < 0 ? 0 : current + delta);
  }

  function onStepListKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        moveSelection(1);
        break;
      case "ArrowUp":
        event.preventDefault();
        moveSelection(-1);
        break;
      case "Home":
        event.preventDefault();
        selectStepAt(0);
        break;
      case "End":
        event.preventDefault();
        selectStepAt(steps.length - 1);
        break;
    }
  }

  /** The filmstrip reorders horizontally, so hit tests use the column rectangles. */
  function collectColRects(): ColRect[] {
    if (!filmstripEl) return [];
    return Array.from(filmstripEl.querySelectorAll<HTMLLIElement>(":scope > li")).map(
      (li) => li.getBoundingClientRect(),
    );
  }

  async function applyStepReorder(
    stepId: string,
    resolveTo: (from: number, length: number) => number | null,
  ) {
    if (reorderBusy) return;
    const previewFrom = steps.findIndex((step) => step.id === stepId);
    if (previewFrom < 0 || resolveTo(previewFrom, steps.length) == null) return;
    reorderBusy = true;
    try {
      // Griff-pointerdown ruft preventDefault, der Textarea-Blur feuert
      // nicht zuverlässig. Entwurf vorher sichern, Index danach neu lesen.
      await onBeforeReorder();
      const from = steps.findIndex((step) => step.id === stepId);
      if (from < 0) return;
      const to = resolveTo(from, steps.length);
      if (to == null) return;
      const next = await onMove(from, to);
      const pos = next.findIndex((step) => step.id === stepId);
      if (pos >= 0) reorderAnnounce = fillCount(copy.stepPosition, pos + 1, next.length);
    } finally {
      reorderBusy = false;
    }
  }

  function onGripPointerDown(event: PointerEvent, stepId: string, index: number) {
    if (event.button !== 0 || reorderBusy || dragActive) return;
    event.preventDefault();
    event.stopPropagation();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    drag = { kind: "dragging", stepId, gap: index };
  }

  function onGripPointerMove(event: PointerEvent) {
    if (!dragActive) return;
    const gap = gapFromClientX(event.clientX, collectColRects());
    if (gap !== dragActive.gap) drag = { ...dragActive, gap };
  }

  function finishGripDrag(event: PointerEvent) {
    if (!dragActive) return;
    const stepId = dragActive.stepId;
    drag = { kind: "idle" };
    const handle = event.currentTarget;
    if (handle instanceof HTMLElement && handle.hasPointerCapture(event.pointerId)) {
      handle.releasePointerCapture(event.pointerId);
    }
    if (event.type !== "pointerup") return;
    const gap = gapFromClientX(event.clientX, collectColRects());
    void applyStepReorder(stepId, (from) => moveTargetForGap(from, gap));
  }

  function onGripKeydown(event: KeyboardEvent, stepId: string) {
    const arrows = ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"];
    if (!event.altKey || !arrows.includes(event.key)) return;
    event.preventDefault();
    event.stopPropagation();
    const dir = event.key === "ArrowUp" || event.key === "ArrowLeft" ? "up" : "down";
    void applyStepReorder(stepId, (from, length) => slotAfterKeyboard(from, dir, length));
  }
</script>

<div class="sl-filmstrip-wrap">
  <div class="sl-filmstrip-header">
    <p class="sl-section-label">{copy.filmstrip}</p>
    <p class="sl-filmstrip-meta">{countLabel} · {copy.filmstripMeta}</p>
  </div>
  <ol class="sl-filmstrip" bind:this={filmstripEl} aria-label={copy.filmstrip}>
    {#each steps as step, index (step.id)}
      {@const thumbSrc = imageById[step.id] ?? null}
      <li
        class:drop-before={dragActive?.gap === index}
        class:drop-after={dragActive?.gap === steps.length && index === steps.length - 1}
        class:dragging-source={dragActive?.stepId === step.id}
      >
        <button
          type="button"
          class="sl-film-grip"
          class:grabbing={dragActive?.stepId === step.id}
          aria-label={copy.gripLabel}
          title={copy.gripLabel}
          aria-roledescription={copy.gripRole}
          onpointerdown={(event) => onGripPointerDown(event, step.id, index)}
          onpointermove={onGripPointerMove}
          onpointerup={finishGripDrag}
          onpointercancel={finishGripDrag}
          onkeydown={(event) => onGripKeydown(event, step.id)}
        ><GripVertical class="sl-icon" /></button>
        <button
          type="button"
          class="sl-film-frame"
          class:selected={step.id === selectedId}
          data-step-id={step.id}
          onclick={() => (selectedId = step.id)}
          onkeydown={onStepListKeydown}
        >
          <span class="sl-film-number">{index + 1}</span>
          <span class="sl-film-thumb">
            {#if thumbSrc}
              <img
                src={thumbSrc}
                alt=""
                draggable="false"
                class:cropped={step.crop != null}
                style:width={step.crop ? `${100 / step.crop.w}%` : undefined}
                style:height={step.crop ? `${100 / step.crop.h}%` : undefined}
                style:transform={step.crop
                  ? `translate(${-step.crop.x * 100}%, ${-step.crop.y * 100}%)`
                  : undefined}
              />
            {:else}
              <span class="sl-film-placeholder" aria-hidden="true"></span>
            {/if}
          </span>
        </button>
      </li>
    {/each}
  </ol>
  <p class="sl-sr-only" aria-live="polite">{reorderAnnounce}</p>
</div>
