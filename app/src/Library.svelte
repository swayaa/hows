<script lang="ts">
  import "./styles/library.css";
  import BookOpen from "@lucide/svelte/icons/book-open";
  import CircleDot from "@lucide/svelte/icons/circle-dot";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Settings from "@lucide/svelte/icons/settings";
  import EmptyState from "./EmptyState.svelte";
  import type { Messages } from "./locales";
  import type { RecentGuideEntry } from "./openGuide";
  import RecentGuideCard from "./RecentGuideCard.svelte";

  let {
    copy,
    localeCode,
    recentGuides,
    captureHotkey,
    onStart,
    onOpen,
    onSettings,
  }: {
    copy: Messages;
    /** UI language, for the dates on the cards. */
    localeCode: string;
    recentGuides: RecentGuideEntry[];
    /** Live hotkey once settings loaded, default until then. */
    captureHotkey: string;
    onStart: () => void;
    /** `null` opens the native file dialog. */
    onOpen: (path: string | null) => void;
    onSettings: () => void;
  } = $props();
</script>

<section class="sl-library">
  <div class="sl-library-top">
    <button class="sl-ghost-btn" type="button" data-action="open-settings" onclick={onSettings}>
      <Settings class="sl-icon" />{copy.settings}
    </button>
  </div>

  <div class="sl-hero">
    <h1 class="sl-hero-title">{copy.libraryH1}</h1>
    <p class="sl-hero-sub">{copy.librarySub}</p>
    <div class="sl-hero-actions">
      <button class="sl-btn sl-btn-primary sl-cta-primary" type="button" data-action="new-capture" onclick={onStart}>
        <CircleDot class="sl-icon" />
        <span>{copy.newCapture}</span>
        <span class="sl-cta-hotkey">{captureHotkey}</span>
      </button>
      <div class="sl-cta-secondary">
        <button class="sl-btn sl-btn-outlined" type="button" data-action="open-file" onclick={() => onOpen(null)}>
          <FolderOpen class="sl-icon" />{copy.open}
        </button>
        <span class="sl-cta-micro">{copy.openMicro}</span>
      </div>
    </div>
  </div>

  {#if recentGuides.length > 0}
    <div class="sl-recent">
      <div class="sl-recent-header">
        <p class="sl-section-label">{copy.recent}</p>
      </div>
      <ul class="sl-recent-list">
        {#each recentGuides as entry (entry.path)}
          <li><RecentGuideCard {copy} {localeCode} {entry} onOpen={onOpen} /></li>
        {/each}
      </ul>
    </div>
  {:else}
    <EmptyState icon={BookOpen} title={copy.emptyLibraryTitle} hint={copy.emptyLibraryHint} />
  {/if}
</section>
