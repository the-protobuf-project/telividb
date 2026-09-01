<!--
  The page frame every panel renders through.

  It exists because panels were each building this by hand, and they drifted:
  measured at 1180, the seven panels put their heading at five different x
  positions across four root grids, so switching panels moved the title by up to
  272px. The frame is not a convenience wrapper — it is the thing that makes
  "the heading is always in the same place" true by construction rather than by
  everyone remembering.

  Two rules it enforces, which is most of the reason it is a component:

  1. **The head never scrolls and the body never grows past the pane.** The
     frame is `auto 1fr` inside a `min-height: 0` grid, and the body is
     `overflow: hidden`. A panel whose content does not fit does not get a
     scrollbar — it gets redesigned, paginated, or given an explicit
     `<DataViewport>` for the one region that is genuinely unbounded.
  2. **One gutter.** The body starts one `--gutter` from its column, exactly
     where the head does. Centring the content on a fixed measure is what broke
     Models: `deviations.css` sets `.page-inner { max-width: none }` on purpose,
     because the window is not a document and it has the width.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** The page's name. Rendered as the one `h1` in the panel. */
    title: string;
    /**
     * A sentence under the heading saying what this page is for.
     *
     * Optional, and it should stay one or two lines — it sits in the head,
     * which is fixed height, so a paragraph here is height the body loses.
     */
    lede?: string;
    /** Controls that act on the whole page, pinned to the trailing edge. */
    actions?: Snippet;
    /** The page itself. */
    children: Snippet;
  }

  let { title, lede, actions, children }: Props = $props();
</script>

<div class="page">
  <div class="page-top">
    <div class="page-inner">
      <div class="page-head">
        <h1>{title}</h1>
        <span class="spacer"></span>
        {#if actions}{@render actions()}{/if}
      </div>
      {#if lede}<p class="lede">{lede}</p>{/if}
    </div>
  </div>

  <div class="page-body">
    <div class="page-inner">
      {@render children()}
    </div>
  </div>
</div>
