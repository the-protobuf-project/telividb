<!--
  The one region on a page that is allowed to scroll.

  A panel must fit its pane — that is the rule, and `Page` enforces it by
  clipping. But some content is genuinely unbounded: ten thousand points, a
  conversation of two hundred turns, a catalogue that grows. Those get this,
  which is a *bounded* window with its own scrollbar, sized by what is left
  after the head and whatever is pinned below it.

  The distinction that matters: the page does not scroll, this does. A header
  stays put, a pager stays put, and only the rows move — so the controls never
  travel off the bottom of the window, which is what a scrolling page does to
  them.

  Reach for it deliberately. If a panel wants one because it has too many
  *sections*, that is a panel to redesign, not a panel to wrap.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /**
     * What a screen reader calls this region.
     *
     * A scrollable box is focusable and lands in the tab order, so it needs a
     * name like any other stop — without one it announces as an unlabelled
     * group and gives no hint what is inside.
     */
    label: string;
    /** The rows, turns, or whatever is genuinely unbounded. */
    children: Snippet;
  }

  let { label, children }: Props = $props();
</script>

<!-- `tabindex="0"` is deliberate rather than accidental: a region that scrolls
     must be reachable by keyboard, or its content is only readable with a
     pointer. Firefox and Chrome both give a scrollable div keyboard scrolling
     once it can hold focus.

     The rule below fires on "noninteractive element with a nonnegative
     tabindex", which is the right warning for a div someone made clickable and
     the wrong one here: WCAG 2.1.1 requires exactly this for scrollable
     content, and `role="region"` plus a name is what makes it announce as a
     landmark rather than as a control. Suppressed with the reason rather than
     worked around, because every alternative is worse ARIA. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="data-viewport" role="region" aria-label={label} tabindex="0">
  {@render children()}
</div>
