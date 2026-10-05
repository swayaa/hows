<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import FileText from "@lucide/svelte/icons/file-text";
  import FileX from "@lucide/svelte/icons/file-x";
  import type { Messages } from "./locales";
  import { recentMeta, recentTitle, type RecentGuideEntry } from "./openGuide";

  let {
    copy,
    localeCode,
    entry,
    onOpen,
  }: {
    copy: Messages;
    localeCode: string;
    entry: RecentGuideEntry;
    onOpen: (path: string) => void;
  } = $props();

  let thumb: HTMLSpanElement | undefined = $state();
  let thumbnail = $state<string | null>(null);
  const unavailable = $derived(entry.missing || entry.unreadable);

  /** Asks for the first screenshot at the size the card shows it, once the card is on screen. */
  $effect(() => {
    const path = entry.path;
    if (!thumb || unavailable || entry.step_count === 0) return;
    const width = Math.round(thumb.clientWidth * window.devicePixelRatio);
    let cancelled = false;
    invoke<string | null>("get_guide_thumbnail", { path, width })
      .then((uri) => {
        if (!cancelled) thumbnail = uri;
      })
      .catch(() => {
        if (!cancelled) thumbnail = null;
      });
    return () => {
      cancelled = true;
    };
  });
</script>

<button
  type="button"
  class="sl-recent-card"
  class:missing={unavailable}
  data-action="open-recent"
  title={entry.path}
  onclick={() => onOpen(entry.path)}
>
  <span class="sl-recent-thumb" bind:this={thumb} aria-hidden="true">
    {#if thumbnail}
      <img src={thumbnail} alt="" data-part="recent-thumbnail" />
    {:else if unavailable}
      <FileX class="sl-icon" />
    {:else}
      <FileText class="sl-icon" />
    {/if}
  </span>
  <span class="sl-recent-text">
    <span class="sl-recent-name">{recentTitle(entry)}</span>
    <span class="sl-recent-meta-line" data-part="recent-meta">{recentMeta(entry, copy, localeCode)}</span>
  </span>
</button>
