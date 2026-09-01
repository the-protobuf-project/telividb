<!--
  Make one thing: type a name, get an id.

  The id is derived from the display name and shown rather than hidden, because
  it is the half that cannot be changed afterwards — it becomes the last segment
  of the resource name. Creation is the only moment a person can still object.
-->
<script lang="ts">
  import { Button, Input } from "$lib/ui";
  import { suggestId } from "$lib/api";

  interface Props {
    /** What is being made, for the placeholder and the button. */
    noun: string;
    /** Whether a call is already in flight. */
    busy?: boolean;
    /**
     * Called with the id and the display name.
     *
     * Resolves to whether the resource was created. The field is only cleared on
     * `true`: a refused name that vanished from the box left the person retyping
     * something the server had already rejected once.
     */
    create: (id: string, displayName: string) => Promise<boolean> | boolean;
  }

  let { noun, busy = false, create }: Props = $props();

  let displayName = $state("");
  let id = $derived(suggestId(displayName));

  /**
   * Why the last attempt was refused, or null.
   *
   * The button used to carry this by being disabled, which states that
   * something is wrong without ever saying what. It stays enabled now and the
   * refusal is a sentence beside the field it belongs to.
   */
  let problem = $state<string | null>(null);
  let field = $state<HTMLElement | null>(null);
  let errorId = $derived(`create-${noun}-error`);

  // Typing is the person answering the message, so it goes away as they do.
  $effect(() => {
    void displayName;
    problem = null;
  });

  async function submit() {
    if (busy) return;
    if (id.length === 0) {
      problem = displayName.trim()
        ? `A ${noun} name needs a letter or a digit in it — that one gives an empty id.`
        : `Type a name for the ${noun}.`;
      field?.querySelector("input")?.focus();
      return;
    }
    const created = await create(id, displayName.trim());
    if (created) displayName = "";
  }
</script>

<div>
  <div style="display: flex; gap: calc(var(--u) * 2); align-items: center" bind:this={field}>
    <Input
      bind:value={displayName}
      placeholder="New {noun}…"
      aria-label="New {noun} name"
      invalid={!!problem}
      aria-describedby={problem ? errorId : undefined}
      style="width: 14rem"
      onkeydown={(e) => {
        if (e.key === "Enter") submit();
      }}
    />
    {#if id}
      <span class="mono faint" style="font-size: 0.75rem" title="The permanent id">
        {id}
      </span>
    {/if}
    <!-- Enabled until the request starts. A disabled submit says something is
         wrong and refuses to say what; this one answers when it is pressed. -->
    <Button size="sm" disabled={busy} loading={busy} onclick={submit}>Create</Button>
  </div>

  {#if problem}
    <p id={errorId} class="hint" style="color: var(--red-text); margin-top: var(--u)">
      {problem}
    </p>
  {/if}
</div>
