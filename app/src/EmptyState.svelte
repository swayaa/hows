<script lang="ts">
  import type { Component, Snippet } from "svelte";

  let {
    icon: Icon,
    title,
    hint = "",
    children,
  }: {
    icon: Component<{ class?: string }>;
    title: string;
    hint?: string;
    /** Optional action below the text, usually one button. */
    children?: Snippet;
  } = $props();
</script>

<div class="sl-empty-state" data-part="empty-state">
  <Icon class="sl-empty-state-icon" />
  <p class="sl-empty-state-title">{title}</p>
  {#if hint}<p class="sl-empty-state-hint">{hint}</p>{/if}
  {@render children?.()}
</div>
