<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Steps from "./Steps.svelte";
  import Button from "./Button.svelte";

  const { Story } = defineMeta({ title: "Elements/Steps" });
  const labels = ["Model", "Organization", "Project", "Space"];
  let at = $state(1);
</script>

<!-- Click through to see done, current and unreached. -->
<Story name="Walkable">
  <div style="display: flex; flex-direction: column; gap: 1rem; max-width: 32rem">
    <Steps {labels} current={at} />
    <div style="display: flex; gap: 0.5rem">
      <Button variant="ghost" size="sm" disabled={at === 0} onclick={() => at--}>Back</Button>
      <Button size="sm" disabled={at === labels.length - 1} onclick={() => at++}>Continue</Button>
    </div>
  </div>
</Story>

<Story name="Every position">
  <div style="display: flex; flex-direction: column; gap: 0.875rem">
    {#each labels as _, i (i)}
      <Steps {labels} current={i} />
    {/each}
  </div>
</Story>
