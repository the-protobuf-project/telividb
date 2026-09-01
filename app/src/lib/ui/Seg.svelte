<!--
  A segmented control: a small set of exclusive choices, all visible.

  Used where a dropdown would hide the alternatives — chat versus graph, auto
  versus a pinned device — because seeing what else is available is the point.
-->
<script lang="ts">
  interface Props {
    /** The choices, in order. */
    options: readonly string[];
    /** Which one is chosen. Bindable. */
    value?: string;
    /** What the group as a whole selects. */
    label: string;
    /** Shown instead of the raw value, when the two differ. */
    display?: (option: string) => string;
  }

  let {
    options,
    value = $bindable(""),
    label,
    display = (o: string) => o,
  }: Props = $props();
</script>

<div class="seg" role="group" aria-label={label}>
  {#each options as option (option)}
    <button
      type="button"
      aria-pressed={value === option}
      onclick={() => (value = option)}
    >
      {display(option)}
    </button>
  {/each}
</div>
