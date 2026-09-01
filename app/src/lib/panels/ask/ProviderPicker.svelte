<!--
  Which model writes the answer, and where it runs.

  Locality is on the face of the control rather than in a settings page, because
  it is the one property that changes what happens to the passages: a remote
  provider sends them off this machine. A person choosing between two names
  should not have to remember which is which.
-->
<script lang="ts">
  import * as Select from "$lib/components/ui/select";
  import { Badge } from "$lib/components/ui/badge";
  import type { AskState } from "./state.svelte";

  interface Props {
    /** The panel state this control reads and writes. */
    state: AskState;
  }

  let { state }: Props = $props();

  /** Chosen provider id, as the select binds it. */
  let selected = $derived(state.provider?.id ?? "");

  function choose(id: string) {
    const found = state.providers.find((p) => p.id === id);
    if (found) state.select(found);
  }
</script>

<div class="flex items-center gap-2">
  <Select.Root type="single" value={selected} onValueChange={choose}>
    <Select.Trigger class="w-40">
      {state.provider?.displayName ?? "no provider"}
    </Select.Trigger>
    <Select.Content>
      {#each state.providers as provider (provider.id)}
        <!-- An unconfigured remote provider is listed but not selectable: seeing
             that it exists is how a person learns a key can be added, while
             letting it be chosen would only produce a refusal later. -->
        <Select.Item value={provider.id} disabled={!provider.configured}>
          {provider.displayName}
          {#if !provider.configured}
            <span class="text-muted-foreground ml-1 text-xs">needs a key</span>
          {/if}
        </Select.Item>
      {/each}
    </Select.Content>
  </Select.Root>

  {#if state.provider}
    <Select.Root type="single" bind:value={state.model}>
      <Select.Trigger class="w-48">{state.model || "no model"}</Select.Trigger>
      <Select.Content>
        {#each state.provider.models as model (model)}
          <Select.Item value={model}>{model}</Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>

    <Badge variant={state.provider.locality === "local" ? "outline" : "secondary"}>
      {state.provider.locality === "local" ? "on this machine" : "leaves this machine"}
    </Badge>
  {/if}
</div>
