<!--
  A placeholder for content that has not arrived.

  Blocked out at the height the real thing will take, so the list does not jump
  when it loads. That matters more than it sounds: a row that appears under the
  cursor between hover and click is a row the person did not mean to press.
-->
<script lang="ts">
  interface Props {
    /** How many lines to reserve. */
    lines?: number;
    /** Width of the last line, so a block does not read as a solid rectangle. */
    last?: string;
  }

  let { lines = 1, last = "60%" }: Props = $props();
</script>

<div style="display: flex; flex-direction: column; gap: var(--u)" aria-hidden="true">
  {#each Array(lines) as _, i (i)}
    <div class="skeleton" style="width: {i === lines - 1 && lines > 1 ? last : '100%'}"></div>
  {/each}
</div>
