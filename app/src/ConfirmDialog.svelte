<script lang="ts">
  import "./styles/confirm-dialog.css";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";

  let {
    message,
    confirmLabel,
    cancelLabel,
    onAnswer,
  }: {
    message: string;
    /** Names the destructive action, for example "Discard". */
    confirmLabel: string;
    cancelLabel: string;
    onAnswer: (confirmed: boolean) => void;
  } = $props();

  let dialog: HTMLDialogElement | undefined = $state();

  /** A modal dialog traps focus and starts on the first button, which is Cancel. */
  $effect(() => {
    dialog?.showModal();
  });
</script>

<dialog
  bind:this={dialog}
  class="sl-confirm"
  role="alertdialog"
  aria-labelledby="sl-confirm-message"
  data-part="confirm-dialog"
  oncancel={(event) => {
    event.preventDefault();
    onAnswer(false);
  }}
>
  <div class="sl-confirm-body">
    <TriangleAlert class="sl-icon sl-confirm-icon" />
    <p id="sl-confirm-message" class="sl-confirm-message">{message}</p>
  </div>
  <div class="sl-confirm-actions">
    <button class="sl-btn" type="button" data-action="confirm-cancel" onclick={() => onAnswer(false)}>
      {cancelLabel}
    </button>
    <button
      class="sl-btn sl-btn-danger"
      type="button"
      data-action="confirm-accept"
      onclick={() => onAnswer(true)}
    >
      {confirmLabel}
    </button>
  </div>
</dialog>
