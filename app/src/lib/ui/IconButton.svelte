<!--
  A button that is only an icon.

  Separate from `Button` because it is square, borderless and carries a pressed
  state — a toolbar control rather than a call to action. It requires a `label`:
  an icon-only control with no accessible name is unusable by anyone not looking
  at it.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLButtonAttributes, "aria-label"> {
    /** What the icon means. Required — it is the control's only name. */
    label: string;
    /** Whether the control is currently on. */
    pressed?: boolean;
    /** The icon. */
    children: Snippet;
  }

  let {
    label,
    pressed,
    children,
    class: extra = "",
    type = "button",
    ...rest
  }: Props = $props();
</script>

<button
  class="icon-btn {extra}"
  {type}
  aria-label={label}
  title={label}
  aria-pressed={pressed}
  {...rest}
>
  {@render children()}
</button>
