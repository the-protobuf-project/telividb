<!--
  The first run: four steps on one card.

  Replaces a version that created an organization and dropped straight into the
  app — which looked finished while three of the four steps had never run. Each
  step here does its work and only advances if that work succeeded.
-->
<script lang="ts">
  import { Button, Card, Notice, Stage, Steps } from "$lib/ui";
  import { client } from "$lib/api";
  import { step as slide } from "$lib/motion/motion";
  import { Flow, STEPS } from "./flow.svelte";
  import StepPanes from "./Steps.svelte";

  interface Props {
    /** Called once all four steps have completed. */
    ondone: () => void;
  }

  let { ondone }: Props = $props();

  const flow = new Flow(client);
  void flow.load();

  // `done` is set by the flow rather than by the button, so a failed final call
  // cannot close the window on a space that was never created.
  $effect(() => {
    if (flow.done) ondone();
  });

  /** The pane, so a step change can move it. */
  let pane = $state<HTMLElement | null>(null);
  let seen = $state(0);

  // Direction carries meaning: forward slides in from the right, Back from the
  // left, so the flow has a spatial sense rather than merely swapping contents.
  $effect(() => {
    const at = flow.step;
    if (pane && at !== seen) {
      slide(pane, at > seen ? "forward" : "back");
      seen = at;
    }
  });
</script>

<Stage>
  <div style="width: 100%; max-width: 30rem">
    <Card>
      {#snippet head()}
        <Steps labels={STEPS} current={flow.step} />
      {/snippet}

      <div bind:this={pane}><StepPanes {flow} /></div>

      {#if flow.error}
        <Notice tone="error">{flow.error}</Notice>
      {/if}

      {#snippet foot()}
        <Button variant="ghost" disabled={flow.step === 0} onclick={() => flow.back()}>
          Back
        </Button>
        <div style="flex: 1"></div>
        <Button disabled={flow.blocked} loading={flow.busy} onclick={() => flow.next()}>
          {flow.step === STEPS.length - 1 ? "Open the workspace" : "Continue"}
        </Button>
      {/snippet}
    </Card>
  </div>
</Stage>
