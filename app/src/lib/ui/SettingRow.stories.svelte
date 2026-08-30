<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import SettingGroup from "./SettingGroup.svelte";
  import SettingRow from "./SettingRow.svelte";
  import Seg from "./Seg.svelte";
  import Toggle from "./Toggle.svelte";
  import Input from "./Input.svelte";
  import Button from "./Button.svelte";
  import Dot from "./Dot.svelte";

  const { Story } = defineMeta({ title: "Elements/Setting row" });
  let device = $state("Auto");
  let redact = $state(true);
</script>

<Story name="A group">
  <div style="max-width: 46rem">
    <!-- The legend, once, rather than a badge on every row. -->
    <p class="hint" style="margin-bottom: 0.5rem">
      A <span style="border-left:2px solid var(--green); padding-left:0.375rem">green edge</span>
      marks a provider that runs on this machine — the only kind a vault will use.
    </p>
    <SettingGroup>
      <SettingRow label="Data directory" description="Segments, the write-ahead log, models and metadata.">
        {#snippet control()}
          <span class="mono faint" style="font-size: 0.75rem">~/.telividb/data</span>
        {/snippet}
      </SettingRow>

      <SettingRow label="Compute backend" description="Chosen by detection when the process started.">
        {#snippet control()}
          <Seg options={["Auto", "Metal", "CPU"]} bind:value={device} label="Compute backend" />
        {/snippet}
      </SettingRow>

      <SettingRow
        label="Ollama"
        description="Runs on this machine. The only kind a vault will use."
        mark="green"
        markTitle="Runs on this machine"
      >
        {#snippet control()}
          <Input mono value="http://localhost:11434" />
          <Button size="sm" variant="ghost">Forget</Button>
          <Dot state="live" />
        {/snippet}
      </SettingRow>

      <SettingRow
        label="OpenAI"
        description="Sends the question and the retrieved passages off this machine."
      >
        {#snippet control()}
          <Input mono type="password" value="sk-0000000000" />
          <Button size="sm" variant="ghost">Forget</Button>
          <Dot state="live" />
        {/snippet}
      </SettingRow>

      <SettingRow label="Redact before embedding" description="A vector made from a secret leaks it.">
        {#snippet control()}
          <Toggle bind:pressed={redact} label="Redact before embedding" />
        {/snippet}
      </SettingRow>
    </SettingGroup>
  </div>
</Story>

<!-- A group whose controls have nothing behind them yet. -->
<Story name="Not wired">
  <div style="max-width: 46rem">
    <SettingGroup muted>
      <SettingRow label="Send telemetry" description="No traces leave unless you point them at a collector." tag="not built" tone="amber">
        {#snippet control()}
          <span class="faint" style="font-size: 0.75rem">telemetry.toml decides this.</span>
        {/snippet}
      </SettingRow>
    </SettingGroup>
  </div>
</Story>
