<!--
  The first run, start to finish.

  Three steps, and the count is a consequence rather than a design. A setup
  wizard would normally also ask about encryption, about who the people are, and
  which project this belongs to — none of which have a service behind them yet.
  Collecting answers the app cannot act on would be theatre, so those steps
  arrive with the services rather than ahead of them.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { Button } from "$lib/components/ui/button";
  import Collections from "$lib/panels/collections/Collections.svelte";
  import Icon from "$lib/ui/Icon.svelte";
  import Environment from "./Environment.svelte";
  import Welcome from "./Welcome.svelte";
  import { OnboardingState, STEPS } from "./state.svelte";

  interface Props {
    /** Called once, with the collection created here if there was one. */
    ondone: (collection: string) => void;
  }

  let { ondone }: Props = $props();

  const flow = new OnboardingState(client);
  let created = $state("");

  // Asked as the sequence begins rather than when its step is reached, so the
  // environment is on screen the moment a person gets there.
  $effect(() => {
    flow.detect();
  });

  $effect(() => {
    if (flow.step === "done") ondone(created);
  });
</script>

<div class="flex h-full flex-col">
  <header class="flex items-center gap-3 border-b px-4 py-3">
    <span class="text-sm font-medium tracking-tight">telividb</span>

    <!-- Position, not percentage: three steps is few enough to show as dots,
         and a bar implies a duration nobody can predict. -->
    <div class="ml-auto flex items-center gap-1.5">
      {#each STEPS as step, index (step)}
        <span
          class="size-1.5 rounded-full transition-colors
                 {index <= flow.position ? 'bg-primary' : 'bg-muted-foreground/30'}"
        ></span>
      {/each}
    </div>

    <Button variant="ghost" size="sm" onclick={() => flow.finish()}>
      Skip
    </Button>
  </header>

  {#if flow.error}
    <p
      class="selectable border-destructive/40 bg-destructive/10 text-destructive m-4 rounded-lg border px-3 py-2 text-sm"
    >
      {flow.error}
    </p>
  {/if}

  {#if flow.step === "welcome"}
    <Welcome onnext={() => flow.advance()} />
  {:else if flow.step === "environment"}
    <Environment
      capabilities={flow.capabilities}
      onnext={() => flow.advance()}
    />
  {:else if flow.step === "collection"}
    <div class="flex min-h-0 flex-1 flex-col">
      <div class="space-y-1 px-8 pt-8">
        <h2 class="text-lg font-semibold tracking-tight">
          Somewhere to put things
        </h2>
        <p class="text-muted-foreground text-sm">
          A collection is a set of points sharing one schema. Pick a shape to
          start with — you can make more later, and these are the ones this
          build ships compiled.
        </p>
      </div>

      <div class="min-h-0 flex-1">
        <Collections
          oncreated={(collection) => {
            created = collection;
            flow.advance();
          }}
        />
      </div>

      <div class="flex justify-end border-t px-8 py-3">
        <Button variant="ghost" onclick={() => flow.advance()} class="gap-2">
          Later
          <Icon name="arrow-right" />
        </Button>
      </div>
    </div>
  {/if}
</div>
