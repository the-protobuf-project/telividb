<!--
  An on/off switch.

  Requires a `label` and reports `aria-pressed`, so it is operable and
  announceable without the surrounding row — a switch whose only name is the
  sentence beside it is not a control, it is a decoration.
-->
<script lang="ts">
  interface Props {
    /** Whether it is on. Bindable. */
    pressed?: boolean;
    /** What the switch controls. */
    label: string;
    /** Whether it can be changed. */
    disabled?: boolean;
  }

  let { pressed = $bindable(false), label, disabled = false }: Props = $props();
</script>

<button
  class="toggle"
  type="button"
  aria-pressed={pressed}
  aria-label={label}
  {disabled}
  onclick={() => (pressed = !pressed)}
>
  <i></i>
</button>
