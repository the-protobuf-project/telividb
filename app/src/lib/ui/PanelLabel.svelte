<!--
  A section heading: small caps with a rule running to the right.

  The rule is what separates sections without a border box around each one, which
  is why sections in this design can sit close together without looking crowded.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** The heading. */
    children: Snippet;
    /** An optional control at the right-hand end, before the rule. */
    action?: Snippet;
  }

  let { children, action }: Props = $props();
</script>

<div class="panel-label">
  <span>{@render children()}</span>
  {#if action}{@render action()}{/if}
</div>
