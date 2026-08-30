<!--
  The primitive set, with its geometry stated.

  Note the two sizes for a toggle: it *draws* at the mark tier's 20px so it sits
  in a line of text, and *targets* 24px because it is a control. Those answer
  different questions and conflating them is what produced the first fix here —
  a pseudo-element that painted a larger area without enlarging the hit box,
  which is worse than the original, because the target then lies about its size.

  This page exists because the toggle shipped visibly broken — a 12px knob with
  1px above it and 3px below, sitting in a track whose travel did not mirror its
  start. It looked "about right" at 20px and obviously wrong at 4×, and nothing
  in the process would have caught it, because the geometry had never been
  written down. Numbers here are derived from the module, not chosen to look
  correct, and the zoomed row is how they get checked.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import { Bar, Chip, Dot, IconButton, Seg, Steps, Tag, Toggle } from "./index";

  const { Story } = defineMeta({ title: "Elements/Primitives" });

  let on = $state(true);
  let mode = $state("Auto");

  /** The rules every primitive is built from. */
  const rules = [
    ["Module", "4px", "Every dimension is a whole multiple."],
    ["Form tier", "32px", "Button, input, segmented control, tree node."],
    ["Compact tier", "28px", "Small button, icon button, chip."],
    ["Mark tier", "20px", "Tag, toggle, step number — sits in a line of text."],
    ["Pointer target", "24px", "WCAG 2.2 AA. A mark that is also a target draws at 20 and targets at 24."],
    ["Baseline", "20px", "14px text. Keeps stacked lines on the module."],
    ["Inside a border", "−2px", "A child of a bordered box fits its interior."],
    ["Clearance", "equal", "Top must equal bottom. Travel must mirror the start."],
  ];
</script>

<Story name="Geometry">
  <div style="display: flex; flex-direction: column; gap: 1.25rem; max-width: 52rem">
    <div class="set-group">
      {#each rules as [name, value, why] (name)}
        <div class="set-row">
          <div class="txt"><b>{name}</b><span>{why}</span></div>
          <div class="ctl mono" style="font-size: 0.75rem">{value}</div>
        </div>
      {/each}
    </div>

    <div class="panel-label">At 4×, where geometry shows</div>
    <div style="zoom: 4; display: flex; gap: 0.75rem; align-items: center; width: fit-content">
      <Toggle pressed={false} label="Off" />
      <Toggle pressed label="On" />
      <Tag tone="green">ready</Tag>
      <Dot state="live" />
    </div>

    <div class="panel-label">At 4×, controls</div>
    <div style="zoom: 4; display: flex; gap: 0.75rem; align-items: center; width: fit-content">
      <Seg options={["Auto", "CPU"]} bind:value={mode} label="Backend" />
      <Chip label="metal" state="live" />
      <IconButton label="Add">
        <svg width="14" height="14" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M4 9h10M9 4v10" /></svg>
      </IconButton>
    </div>

    <div class="panel-label">At 4×, marks in text</div>
    <div style="zoom: 4; width: fit-content">
      <Steps labels={["Model", "Space"]} current={1} />
    </div>

    <div class="panel-label">Progress, at every fill</div>
    <div style="display: flex; flex-direction: column; gap: 0.5rem; max-width: 20rem">
      {#each [0, 0.25, 0.64, 1] as v (v)}<Bar value={v} />{/each}
    </div>

    <!-- Bound so the switch can be worked, not only looked at: a control that
         is only ever screenshotted in one state is one whose transition nobody
         has seen. -->
    <div class="panel-label">Working</div>
    <div style="display: flex; gap: 0.75rem; align-items: center">
      <Toggle bind:pressed={on} label="Redact before embedding" />
      <span class="hint">{on ? "on" : "off"} — click it</span>
    </div>
  </div>
</Story>
