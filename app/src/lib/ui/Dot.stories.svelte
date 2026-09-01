<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Dot from "./Dot.svelte";

  const { Story } = defineMeta({ title: "Elements/Dot" });
</script>

<Story name="Every state">
  <div style="display: flex; gap: 1rem; align-items: center">
    {#each ["idle", "live", "warn", "off"] as const as state (state)}
      <span style="display: inline-flex; align-items: center; gap: 0.375rem">
        <Dot {state} /><span class="faint" style="font-size: 0.75rem">{state}</span>
      </span>
    {/each}
  </div>
</Story>
