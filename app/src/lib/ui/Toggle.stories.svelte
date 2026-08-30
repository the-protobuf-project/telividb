<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Toggle from "./Toggle.svelte";
  import Seg from "./Seg.svelte";

  const { Story } = defineMeta({ title: "Elements/Toggle" });
  let on = $state(true);
  let off = $state(false);
  let mode = $state("Auto");
</script>

<!-- Both states side by side, and large, because the geometry is the whole
     point: a knob that does not sit centred reads as broken at any size. -->
<Story name="Both states">
  <div style="display: flex; flex-direction: column; gap: 1.25rem">
    <div style="display: flex; gap: 1.5rem; align-items: center">
      <span style="display: inline-flex; align-items: center; gap: 0.5rem">
        <Toggle bind:pressed={off} label="Off example" />
        <span class="hint">off</span>
      </span>
      <span style="display: inline-flex; align-items: center; gap: 0.5rem">
        <Toggle bind:pressed={on} label="On example" />
        <span class="hint">on</span>
      </span>
      <span style="display: inline-flex; align-items: center; gap: 0.5rem">
        <Toggle pressed disabled label="Disabled example" />
        <span class="hint">disabled</span>
      </span>
    </div>

    <!-- Enlarged four times, which is how the misalignment was spotted. -->
    <div>
      <p class="hint" style="margin-bottom: 0.5rem">At 4×, where the geometry shows:</p>
      <div style="zoom: 4; display: flex; gap: 0.75rem; width: fit-content">
        <Toggle pressed={false} label="Zoomed off" />
        <Toggle pressed label="Zoomed on" />
      </div>
    </div>
  </div>
</Story>

<!-- The other exclusive control, for comparison: a toggle is two states, a
     segmented control is more than two and shows them all. -->
<Story name="Beside a segmented control">
  <div style="display: flex; gap: 1.5rem; align-items: center">
    <Toggle bind:pressed={on} label="Redact before embedding" />
    <Seg options={["Auto", "Metal", "CPU"]} bind:value={mode} label="Compute backend" />
  </div>
</Story>
