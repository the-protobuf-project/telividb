<!--
  A text field.

  `bind:value` works on it, so a caller writes `<Input bind:value={name} />` and
  not an `oninput` handler — which matters because the design derives ids and
  slugs from what is typed, live.
-->
<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLInputAttributes, "value"> {
    /** The field's contents. Bindable. */
    value?: string;
    /** Render in the monospace face — for ids, hosts, keys. */
    mono?: boolean;
    /**
     * Whether the value is rejected.
     *
     * Marks the field and sets `aria-invalid`, so the state is announced rather
     * than only coloured — a red edge is invisible to a reader who cannot see it
     * and to anyone with a red deficiency.
     */
    invalid?: boolean;
  }

  let {
    value = $bindable(""),
    mono = false,
    invalid = false,
    class: extra = "",
    ...rest
  }: Props = $props();
</script>

<input
  class="input {mono ? 'mono' : ''} {invalid ? 'invalid' : ''} {extra}"
  aria-invalid={invalid}
  bind:value
  {...rest}
/>
