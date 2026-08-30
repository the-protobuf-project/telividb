<!--
  Answering: which models may write an answer, and the keys that reach them.

  A local provider gets a text field because what it holds is an address; a
  remote one gets a password field and shows a mask once stored. The engine never
  hands back what was written — the mask states that something is there, it does
  not render it.
-->
<script lang="ts">
  import { Button, Dot, Input, Notice, SettingGroup, SettingRow } from "$lib/ui";
  import { MASK, type SettingsState } from "./state.svelte";

  interface Props {
    /** The panel state this section reads and writes. */
    state: SettingsState;
  }

  let { state }: Props = $props();
</script>

<div style="display: flex; flex-direction: column; gap: calc(var(--u) * 3)">
  <Notice>
    Retrieval happens on this machine either way. A provider is only what writes
    the answer — and everything it is given is listed on the turn, so nothing is
    sent that you cannot see. A green edge marks one that runs here.
  </Notice>

  <SettingGroup>
    {#each state.providers as provider (provider.id)}
      <SettingRow
        label={provider.displayName}
        description={provider.note}
        mark={provider.locality === "local" ? "green" : undefined}
        markTitle={provider.locality === "local"
          ? "Runs on this machine — the only kind a vault will use"
          : undefined}
      >
        {#snippet control()}
          <Input
            mono
            type={provider.locality === "local" ? "text" : "password"}
            placeholder={provider.credentialHint}
            value={state.shown(provider)}
            aria-label="{provider.displayName} credential"
            style="width: 13rem; font-size: 0.75rem"
            oninput={(e) => (state.drafts[provider.id] = e.currentTarget.value)}
            onfocus={(e) => {
              // The mask is not a value. Clearing it stops someone editing a row
              // of dots into something they meant to be a key.
              if (e.currentTarget.value === MASK) state.drafts[provider.id] = "";
            }}
          />
          {#if state.dirty(provider.id)}
            <Button
              size="sm"
              loading={state.saving === provider.id}
              onclick={() => state.save(provider.id)}
            >
              Save
            </Button>
          {:else if provider.configured && provider.locality === "remote"}
            <Button
              size="sm"
              variant="ghost"
              loading={state.saving === provider.id}
              onclick={() => state.forget(provider.id)}
            >
              Forget
            </Button>
          {/if}
          <Dot
            state={provider.configured ? "live" : "idle"}
            title={provider.configured
              ? `${provider.displayName} is ready`
              : `${provider.displayName} is not configured`}
          />
        {/snippet}
      </SettingRow>
    {/each}
  </SettingGroup>
</div>
