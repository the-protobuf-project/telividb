<!--
  One setting: what it is, what it means, and the control that changes it.

  The description is not decoration. Most settings here decide where data goes,
  and a label alone ("Redact before embedding") tells a reader what the switch is
  called rather than what happens if they touch it.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import Tag from "./Tag.svelte";
  import type { TagTone } from "./Tag.svelte";

  interface Props {
    /** What the setting is called. */
    label: string;
    /** What it means, or what breaks without it. */
    description?: string;
    /** A badge beside the label — "not built", "sample data". */
    tag?: string;
    /** What the badge asserts. */
    tone?: TagTone;
    /**
     * A coloured rule at the row's leading edge.
     *
     * For a property that is true of the whole row rather than of one control —
     * where a provider runs, say. It costs no horizontal space, which a badge
     * does, and a column of them is scannable in one pass: the transit-map
     * approach of colouring the line rather than labelling every stop.
     *
     * Colour alone is never the only signal. Pair it with a legend above the
     * group, and give it a `markTitle` so it is reachable on hover.
     */
    mark?: TagTone;
    /** What the leading rule means, for its tooltip. */
    markTitle?: string;
    /**
     * The control.
     *
     * Optional, because a row is also how an empty state is written — "No
     * projects yet" is a row with nothing on its right, and requiring a snippet
     * there would mean passing an empty one at every call site.
     */
    control?: Snippet;
  }

  let {
    label,
    description,
    tag,
    tone = "plain",
    mark,
    markTitle,
    control,
  }: Props = $props();
</script>

<div class="set-row" data-mark={mark} title={markTitle}>
  <div class="txt">
    <b>
      {label}
      {#if tag}<Tag {tone}>{tag}</Tag>{/if}
    </b>
    {#if description}<span>{description}</span>{/if}
  </div>
  {#if control}<div class="ctl">{@render control()}</div>{/if}
</div>
