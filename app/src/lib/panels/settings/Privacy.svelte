<!--
  Privacy, and the controls that do not exist yet.

  Every row is disabled and says what it is waiting on. Three switches that
  looked settable and did nothing would be worse than none: a person who thinks
  they turned redaction on has been told something false about where their data
  goes, and this is the screen where that matters most.
-->
<script lang="ts">
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

<div>
  <div class="panel-label">Privacy</div>
  <div class="set-group" style="margin-top: 0.625rem; opacity: 0.6">
    {#each pending as row (row.label)}
      <div class="set-row">
        <div class="txt">
          <b>{row.label} <span class="tag">not built</span></b>
          <span>{row.description}</span>
        </div>
        <div class="ctl faint" style="font-size: 0.75rem; max-width: 14rem; text-align: right">
          {row.waiting}
        </div>
      </div>
    {/each}
  </div>
</div>
