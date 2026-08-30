<!--
  Answering: which models may write an answer, and the keys that reach them.

  A local provider gets a text field because what it holds is an address; a
  remote one gets a password field and shows a mask once stored. The engine never
  hands back what was written — the mask states that something is there, it does
  not render it.
-->
<script lang="ts">
  import { MASK, type SettingsState } from "./state.svelte";

  interface Props {
    /** The panel state this section reads and writes. */
    state: SettingsState;
  }

  let { state }: Props = $props();
</script>

<div>
  <p class="hint" style="margin: 0.5rem 0 0.625rem">
    Retrieval happens on this machine either way. A provider is only what writes
    the answer — and everything it is given is listed on the turn, so nothing is
    sent that you cannot see.
  </p>

  <div class="set-group">
    {#each state.providers as provider (provider.id)}
      <div class="set-row">
        <div class="txt">
          <b>
            {provider.displayName}
            {#if provider.locality === "local"}
              <span class="tag green">local</span>
            {/if}
          </b>
          <span>{provider.note}</span>
        </div>
        <div class="ctl" style="display: flex; gap: 0.5rem; align-items: center">
          <input
            class="input mono"
            style="width: 13rem; font-size: 0.75rem"
            type={provider.locality === "local" ? "text" : "password"}
            placeholder={provider.credentialHint}
            value={state.shown(provider)}
            aria-label="{provider.displayName} credential"
            oninput={(e) => (state.drafts[provider.id] = e.currentTarget.value)}
            onfocus={(e) => {
              // The mask is not a value. Clearing it stops someone editing a row
              // of dots into something they meant to be a key.
              if (e.currentTarget.value === MASK) state.drafts[provider.id] = "";
            }}
          />
          {#if state.dirty(provider.id)}
            <button
              class="btn sm"
              type="button"
              disabled={state.saving === provider.id}
              onclick={() => state.save(provider.id)}
            >
              {state.saving === provider.id ? "Saving…" : "Save"}
            </button>
          {:else if provider.configured && provider.locality === "remote"}
            <button
              class="btn ghost sm"
              type="button"
              disabled={state.saving === provider.id}
              onclick={() => state.forget(provider.id)}
            >
              Forget
            </button>
          {/if}
          <span
            class="dot"
            class:live={provider.configured}
            title={provider.configured ? "Ready" : "Not configured"}
          ></span>
        </div>
      </div>
    {/each}
  </div>
</div>
