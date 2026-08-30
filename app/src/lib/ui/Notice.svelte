<!--
  A message that is not a field's: a refusal, a warning, a fact worth stating
  before an action.

  A left rule rather than a filled panel — the colour carries the whole signal,
  and a block of tinted background at the top of a page competes with the content
  it is about.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  /** How much the message wants attention. */
  export type NoticeTone = "info" | "ok" | "warn" | "error";

  interface Props {
    /** How much the message wants attention. */
    tone?: NoticeTone;
    /** The message. */
    children: Snippet;
  }

  let { tone = "info", children }: Props = $props();
</script>

<p
  class="notice {tone === 'info' ? '' : tone} selectable"
  role={tone === "error" ? "alert" : undefined}
>
  {@render children()}
</p>
