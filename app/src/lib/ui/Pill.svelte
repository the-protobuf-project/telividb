<!--
  A removable selection: an attached file, a chosen connector, a filter.

  Distinct from `Tag` in that a pill is something the person put there and can
  take away, which is why it carries a dismiss control and a pressed state.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** Whether it is active. */
    pressed?: boolean;
    /** Called when the dismiss control is used. Omit for a fixed pill. */
    onremove?: () => void;
    /** Clicking the pill itself. */
    onclick?: () => void;
    /** A status dot before the label. */
    dot?: Snippet;
    /** The label. */
    children: Snippet;
  }

  let { pressed, onremove, onclick, dot, children }: Props = $props();
</script>

<button class="pill" type="button" aria-pressed={pressed} {onclick}>
  {#if dot}{@render dot()}{/if}
  <span>{@render children()}</span>
  {#if onremove}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <span
      class="x"
      role="button"
      tabindex="0"
      aria-label="Remove"
      onclick={(e) => {
        e.stopPropagation();
        onremove();
      }}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          e.stopPropagation();
          onremove();
        }
      }}
    >×</span>
  {/if}
</button>
