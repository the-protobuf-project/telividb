<!--
  Answering: which models may write an answer, and the keys that reach them.

  A local provider gets a plain text field because what it holds is an address,
  not a secret; a remote one gets a password field and shows a mask once stored.
  The engine never hands back what was written — the mask is a statement that
  something is there, not a rendering of it.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import SettingRow from "$lib/ui/SettingRow.svelte";
  import { MASK, type SettingsState } from "./state.svelte";

  interface Props {
    /** The panel state this section reads and writes. */
    state: SettingsState;
  }

  let { state }: Props = $props();
</script>

<section class="flex flex-col gap-2">
  <h2 class="text-muted-foreground text-xs tracking-wide uppercase">Answering</h2>
  <p class="text-muted-foreground max-w-prose text-xs">
    Retrieval happens on this machine either way. A provider is only what writes
    the answer — and everything it is given is listed on the turn, so nothing is
    sent that you cannot see.
  </p>

  <div class="border-border mt-1 border">
    {#each state.providers as provider (provider.id)}
      <SettingRow
        label={provider.displayName}
        description={provider.note}
        tag={provider.locality === "local" ? "local" : undefined}
      >
        {#snippet control()}
          <Input
            type={provider.locality === "local" ? "text" : "password"}
            placeholder={provider.credentialHint}
            value={state.shown(provider)}
            oninput={(e) => {
              state.drafts[provider.id] = e.currentTarget.value;
            }}
            onfocus={(e) => {
              // The mask is not a value. Clearing it on focus stops someone
              // editing a row of dots into something they meant to be a key.
              if (e.currentTarget.value === MASK) state.drafts[provider.id] = "";
            }}
            class="w-52 font-mono text-xs"
            aria-label="{provider.displayName} credential"
          />
          {#if state.dirty(provider.id)}
            <Button
              size="sm"
              disabled={state.saving === provider.id}
              onclick={() => state.save(provider.id)}
            >
              {state.saving === provider.id ? "Saving…" : "Save"}
            </Button>
          {:else if provider.configured && provider.locality === "remote"}
            <Button
              size="sm"
              variant="ghost"
              disabled={state.saving === provider.id}
              onclick={() => state.forget(provider.id)}
            >
              Forget
            </Button>
          {/if}
          <span
            class="size-1.5 rounded-full {provider.configured
              ? 'bg-[var(--color-green)]'
              : 'bg-[var(--color-rule-strong)]'}"
            title={provider.configured ? "Ready" : "Not configured"}
          ></span>
        {/snippet}
      </SettingRow>
    {/each}
  </div>
</section>
