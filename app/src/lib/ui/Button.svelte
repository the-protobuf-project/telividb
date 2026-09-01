<!--
  The button.
  
  Inverted by default — a light fill on the dark ground — because the design
  gives its one strong action the highest contrast on screen. `ghost` is the
  secondary: an outline that only gains a border on hover.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  /** How much weight the action carries. */
  export type ButtonVariant = "primary" | "ghost";
  /** Standard, or the compact form used in dense rows and headers. */
  export type ButtonSize = "md" | "sm";

  interface Props extends HTMLButtonAttributes {
    /** How much weight the action carries. */
    variant?: ButtonVariant;
    /** Standard, or the compact form. */
    size?: ButtonSize;
    /**
     * Whether the action is in flight.
     *
     * Disables the button as well as marking it: a second click during a create
     * is how two organizations get made from one intention.
     */
    loading?: boolean;
    /** The label. */
    children: Snippet;
  }

  let {
    variant = "primary",
    size = "md",
    loading = false,
    children,
    class: extra = "",
    type = "button",
    disabled = false,
    ...rest
  }: Props = $props();

  let classes = $derived(
    ["btn", variant === "ghost" && "ghost", size === "sm" && "sm", extra]
      .filter(Boolean)
      .join(" "),
  );
</script>

<button
  class={classes}
  {type}
  disabled={disabled || loading}
  data-loading={loading}
  aria-busy={loading}
  {...rest}
>
  {@render children()}
</button>
