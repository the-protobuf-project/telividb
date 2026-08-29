<!--
  The first thing a person sees.

  One mark, one sentence, one button. The launch sequence already brings the
  window in; this step exists so the app says what it is before it starts asking
  for decisions.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Icon from "$lib/ui/Icon.svelte";

  interface Props {
    /** Continue to the next step. */
    onnext: () => void;
  }

  let { onnext }: Props = $props();
</script>

<div class="flex flex-1 flex-col items-center justify-center gap-6 p-8 text-center">
  <div
    class="bg-primary/10 text-primary flex size-16 items-center justify-center rounded-2xl"
  >
    <Icon name="layers-1" class="text-3xl" label="telividb" />
  </div>

  <div class="space-y-2">
    <h1 class="text-2xl font-semibold tracking-tight">telividb</h1>
    <p class="text-muted-foreground max-w-sm text-sm">
      A vector and graph database that runs on this machine. Your data stays
      here — the engine is in this window, not behind a service.
    </p>
  </div>

  <Button onclick={onnext} class="gap-2">
    Set up
    <Icon name="arrow-right" />
  </Button>
</div>
