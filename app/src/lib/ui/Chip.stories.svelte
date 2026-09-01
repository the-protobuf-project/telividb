<!--
  Every state a chip has, side by side.

  Together rather than one per story, because the question a reader actually has
  is whether these are distinguishable from each other — which a story showing
  one at a time cannot answer.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Chip from "./Chip.svelte";

  const { Story } = defineMeta({
    title: "Readouts/Chip",
    component: Chip,
    argTypes: {
      state: { control: "select", options: ["live", "warn", "off", "idle"] },
    },
  });
</script>

<Story name="Every state">
  <div class="flex flex-wrap items-center gap-2">
    <Chip label="metal" state="live" title="The backend this engine selected" />
    <Chip label="BGE Small" state="live" title="The model held in memory" />
    <Chip label="loading" state="warn" title="Reading weights onto the device" />
    <Chip label="unreachable" state="off" title="The engine did not answer" />
    <Chip label="backend unknown" title="The window has not been able to ask yet" />
  </div>
</Story>

<Story name="One" args={{ label: "metal", state: "live" }} />
