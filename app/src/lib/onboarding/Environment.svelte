<!--
  What this machine turned out to be.

  Detected and shown, never asked. The backend a build *selected* is the one
  fact no orchestrator can see from outside the process — a build that quietly
  fell back to the CPU passes every test while delivering none of the speed —
  so the setup that would normally ask a person to choose instead tells them
  what was found.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import Icon from "$lib/ui/Icon.svelte";
  import type { Capabilities } from "$lib/api";
  import { bytes } from "./state.svelte";

  interface Props {
    /** What the engine reported, or null while it is still being asked. */
    capabilities: Capabilities | null;
    /** Continue to the next step. */
    onnext: () => void;
  }

  let { capabilities, onnext }: Props = $props();

  const accelerated = $derived(
    capabilities !== null && capabilities.environment.backend !== "cpu",
  );
</script>

<div class="flex flex-1 flex-col gap-5 p-8">
  <div class="space-y-1">
    <h2 class="text-lg font-semibold tracking-tight">This machine</h2>
    <p class="text-muted-foreground text-sm">
      Detected at startup. Nothing here is a choice — it is what the engine
      found and is already using.
    </p>
  </div>

  {#if capabilities}
    <dl class="grid gap-3 sm:grid-cols-2">
      <div class="rounded-lg border p-3">
        <dt class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <Icon name="bolt" />
          Compute backend
        </dt>
        <dd class="mt-1.5 flex items-center gap-2">
          <span class="font-mono text-sm">{capabilities.environment.backend}</span>
          <Badge variant={accelerated ? "default" : "secondary"} class="text-[10px]">
            {accelerated ? "accelerated" : "host"}
          </Badge>
        </dd>
        {#if capabilities.environment.budgetSource === "configured"}
          <p class="text-muted-foreground mt-1.5 text-xs">
            Pinned by <span class="font-mono">TELIVIDB_DEVICE</span>, not
            detected.
          </p>
        {/if}
      </div>

      <div class="rounded-lg border p-3">
        <dt class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <Icon name="dashboard-square-1" />
          Device memory
        </dt>
        <dd class="tnum mt-1.5 text-sm">
          <!-- The ceiling this process will use, not the device's free memory.
               The engine reports a budget; free memory is a number about the
               whole machine, and a window that showed it here would be
               answering a different question than the one the label asks. -->
          {#if capabilities.environment.budgetLimitBytes === 0}
            <span class="text-muted-foreground">
              not reported — the host has no separate device memory
            </span>
          {:else}
            {bytes(capabilities.environment.budgetLimitBytes)}
            <span class="text-muted-foreground text-xs">
              ({capabilities.environment.budgetSource})
            </span>
          {/if}
        </dd>
      </div>

      <div class="rounded-lg border p-3 sm:col-span-2">
        <dt class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <Icon name="folder-1" />
          Data directory
        </dt>
        <dd class="selectable mt-1.5 font-mono text-xs break-all">
          {capabilities.data_dir}
        </dd>
        <p class="text-muted-foreground mt-1.5 text-xs">
          Segments, the write-ahead log and metadata. One engine owns it at a
          time — a second window on the same directory is refused rather than
          allowed to corrupt it. Change it with
          <span class="font-mono">TELIVIDB_DATA_DIR</span>.
        </p>
      </div>

      {#if !capabilities.has_model}
        <div class="border-primary/30 bg-primary/5 rounded-lg border p-3 sm:col-span-2">
          <p class="flex items-start gap-2 text-xs">
            <Icon name="circle-question-mark" class="text-primary mt-0.5" />
            <span>
              No embedding model is loaded, so the engine cannot turn text into
              a vector — for storage as much as for search. Point
              the Models panel at a GGUF file to
              enable both. Everything else works without one.
            </span>
          </p>
        </div>
      {/if}
    </dl>
  {:else}
    <p class="text-muted-foreground text-sm">Asking the engine…</p>
  {/if}

  <div class="mt-auto flex justify-end">
    <Button onclick={onnext} disabled={capabilities === null} class="gap-2">
      Continue
      <Icon name="arrow-right" />
    </Button>
  </div>
</div>
