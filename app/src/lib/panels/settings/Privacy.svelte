<!--
  Privacy, and the controls that do not exist yet.

  Every row here is disabled and says what it is waiting on. Three switches that
  looked settable and did nothing would be worse than none: a person who thinks
  they turned redaction on has been told something false about where their data
  goes, and this is the one screen where that matters most.
-->
<script lang="ts">
  import SettingRow from "$lib/ui/SettingRow.svelte";

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
        "Off. Nothing leaves this machine unless you point it at a collector.",
      waiting: "telemetry.toml decides this today, not the window.",
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

<section class="flex flex-col gap-2">
  <h2 class="text-muted-foreground text-xs tracking-wide uppercase">Privacy</h2>

  <div class="border-border border opacity-60">
    {#each pending as row (row.label)}
      <SettingRow label={row.label} description={row.description} tag="not built">
        {#snippet control()}
          <span class="text-muted-foreground max-w-56 text-right text-xs">
            {row.waiting}
          </span>
        {/snippet}
      </SettingRow>
    {/each}
  </div>
</section>
