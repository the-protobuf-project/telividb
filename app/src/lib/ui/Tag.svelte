<!--
  A small bordered label: a state, a licence, a category.

  Colour is a claim rather than decoration — `green` says ready, `amber` says
  attention, `blue` says recommended — so the default is uncoloured and a tone is
  chosen deliberately.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  /** What the colour asserts. */
  export type TagTone = "plain" | "green" | "amber" | "blue";

  interface Props {
    /** What the colour asserts. */
    tone?: TagTone;
    /** Hover text, for the detail the label has no room for. */
    title?: string;
    /** The label. */
    children: Snippet;
  }

  let { tone = "plain", title, children }: Props = $props();
</script>

<span class="tag {tone === 'plain' ? '' : tone}" {title}>{@render children()}</span>
