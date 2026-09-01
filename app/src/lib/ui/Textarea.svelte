<!--
  A multi-line field.

  Non-resizable by design: it lives inside a composer whose height the layout
  owns, and a hand-dragged corner would break the grid it sits in.
-->
<script lang="ts">
  import type { HTMLTextareaAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLTextareaAttributes, "value"> {
    /** The field's contents. Bindable. */
    value?: string;
  }

  let { value = $bindable(""), class: extra = "", rows = 3, ...rest }: Props = $props();
</script>

<textarea class="textarea {extra}" bind:value {rows} {...rest}></textarea>
