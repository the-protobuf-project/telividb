<!--
  Page N of M, with the two steps either side.

  Numbered pages rather than infinite scroll: these lists are things a person
  returns to — "the model was on the second page" is a usable memory, and a
  scroll position is not. It also means the end of the list is reachable, which
  an infinite one never quite is.
-->
<script lang="ts">
  import Button from "./Button.svelte";

  interface Props {
    /** Which page is showing, zero-based. */
    page: number;
    /** How many there are. */
    pages: number;
    /** Step by one, in either direction. */
    go: (delta: number) => void;
  }

  let { page, pages, go }: Props = $props();
</script>

<div
  style="display: flex; align-items: center; gap: calc(var(--u) * 2)"
  role="navigation"
  aria-label="Pages"
>
  <Button variant="ghost" size="sm" disabled={page === 0} onclick={() => go(-1)}>
    Previous
  </Button>
  <span class="hint mono" aria-live="polite">{page + 1} of {pages}</span>
  <Button variant="ghost" size="sm" disabled={page >= pages - 1} onclick={() => go(1)}>
    Next
  </Button>
</div>
