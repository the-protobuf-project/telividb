<!--
  Where a question is asked.

  The provider sits *in* the composer rather than in settings, because it is a
  per-question decision with a consequence — a remote provider is sent the
  passages — and the moment before sending is the only one where changing it
  helps.
-->
<script lang="ts">
  import { Button, Dot, Pill, Textarea } from "$lib/ui";
  import type { Provider } from "@telividb/answer";

  interface Props {
    /** What is typed. Bindable. */
    draft?: string;
    /** Providers this build knows. */
    providers: readonly Provider[];
    /** Which one answers. Bindable. */
    provider?: Provider | null;
    /** Which of its models. Bindable. */
    model?: string;
    /** Whether a question is in flight. */
    busy?: boolean;
    /** Whether text can be embedded at all. */
    canEmbed?: boolean;
    /** Send it. */
    onsend: () => void;
  }

  let {
    draft = $bindable(""),
    providers,
    provider = $bindable(null),
    model = $bindable(""),
    busy = false,
    canEmbed = true,
    onsend,
  }: Props = $props();

  let open = $state(false);
  let ready = $derived(draft.trim().length > 0 && !busy && canEmbed);

  /** Enter sends; shift-enter is a newline, as every chat box behaves. */
  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (ready) onsend();
    }
  }
</script>

<div class="composer">
  <div class="composer-inner">
    <div class="composer-box">
      <Textarea
        bind:value={draft}
        {onkeydown}
        rows={2}
        disabled={!canEmbed}
        placeholder={canEmbed
          ? "Ask anything — every sentence is embedded and kept…"
          : "Waiting for an embedding model to finish loading…"}
      />

      <div class="composer-tools">
        <!-- The provider is named on the composer at all times, not only when
             the menu is open: what happens to the passages depends on it. -->
        <div class="anchor">
          <Pill pressed={open} onclick={() => (open = !open)}>
            {#snippet dot()}
              <Dot
                state={provider ? (provider.locality === "local" ? "live" : "warn") : "idle"}
                title={provider
                  ? provider.locality === "local"
                    ? "runs on this machine"
                    : "sends the passages off this machine"
                  : "no provider — retrieval only"}
              />
            {/snippet}
            {provider ? `${provider.displayName} · ${model}` : "Retrieval only"}
          </Pill>

          {#if open}
            <div class="menu">
              <div class="menu-head">Who writes the answer</div>
              {#each providers as p (p.id)}
                {#each p.models as m (m)}
                  <button
                    class="menu-item"
                    type="button"
                    disabled={!p.configured}
                    onclick={() => {
                      provider = p;
                      model = m;
                      open = false;
                    }}
                  >
                    <Dot state={provider?.id === p.id && model === m ? "live" : "idle"} decorative />
                    <span>{m}</span>
                    <span class="sub">
                      {p.displayName}{p.configured ? "" : " · needs a key"}
                      {p.locality === "local" ? " · local" : ""}
                    </span>
                  </button>
                {/each}
              {/each}
              <button
                class="menu-item"
                type="button"
                style="border-top: 1px solid var(--rule); margin-top: var(--u)"
                onclick={() => {
                  provider = null;
                  open = false;
                }}
              >
                <Dot state={provider ? "idle" : "live"} decorative />
                <span>Retrieval only</span>
                <span class="sub">no answer written</span>
              </button>
            </div>
          {/if}
        </div>

        <span class="spacer" style="flex: 1"></span>
        <span class="hint">Enter to send · Shift+Enter for a new line</span>
        <Button size="sm" disabled={!ready} loading={busy} onclick={onsend}>Ask</Button>
      </div>
    </div>
  </div>
</div>
