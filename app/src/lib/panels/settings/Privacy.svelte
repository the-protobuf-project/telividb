<!--
  Privacy, and the controls that do not exist yet.

  Every row is disabled and says what it is waiting on. Three switches that
  looked settable and did nothing would be worse than none: a person who thinks
  they turned redaction on has been told something false about where their data
  goes, and this is the screen where that matters most.
-->
<script lang="ts">
  import { Notice, SettingGroup, SettingRow } from "$lib/ui";

  /** A setting that is designed but has nothing behind it yet. */
  interface Pending {
    /** What it will be called. */
    readonly label: string;
    /** What it will do. */
    readonly description: string;
    /** Which piece of the system has to exist first. */
    readonly waiting: string;
  }

  const pending: readonly Pending[] = [
    {
      label: "Send telemetry",
      description:
        "Off. No traces or metrics leave this machine unless you point them at a collector.",
      waiting: "telemetry.toml decides this, not the window.",
    },
    {
      label: "Unlock with face or voice",
      description:
        "Recognition would release a key already on this machine. It never authorizes on its own.",
      waiting: "Needs the policy engine and a vault to unlock.",
    },
    {
      label: "Redact before embedding",
      description:
        "Sensitive spans removed before a model sees them — a vector made from a secret leaks it.",
      waiting: "Needs the redaction pass in the ingest path.",
    },
  ];
</script>

<div style="display: flex; flex-direction: column; gap: calc(var(--u) * 3)">
  <!-- Stated before the switches, because it is the one thing on this screen
       that is true right now: a configured remote provider receives the question
       and the passages retrieved for it. Retrieval itself never leaves. -->
  <Notice>
    Retrieval always happens on this machine. Answering does not: a remote
    provider configured under Answering is sent the question and the passages
    found for it, and is shown on every turn before you send.
  </Notice>

  <SettingGroup muted>
    {#each pending as row (row.label)}
      <SettingRow
        label={row.label}
        description={row.description}
        tag="not built"
        tone="amber"
      >
        {#snippet control()}
          <span class="faint" style="font-size: 0.75rem; max-width: 14rem; text-align: right">
            {row.waiting}
          </span>
        {/snippet}
      </SettingRow>
    {/each}
  </SettingGroup>
</div>
