<!--
  A labelled control with its explanation underneath.

  The hint is part of the field rather than an optional extra: most fields in
  this app decide something permanent — a resource id, a protection level — and
  a label alone says what the box is called, not what happens if you fill it.
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    /** What the field is called. */
    label: string;
    /** What the value means, or what it decides. */
    hint?: string;
    /**
     * Why the value was rejected.
     *
     * Replaces the hint rather than joining it: two lines of small print under a
     * field, one explaining and one complaining, is how a form becomes hard to
     * read at the moment it most needs to be easy.
     */
    error?: string;
    /** The control itself. */
    children: Snippet;
  }

  let { label, hint, error, children }: Props = $props();
</script>

<label class="field">
  <span class="label">{label}</span>
  {@render children()}
  {#if error}
    <span class="hint error" role="alert">{error}</span>
  {:else if hint}
    <span class="hint">{hint}</span>
  {/if}
</label>
