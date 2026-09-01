<!--
  One model in the catalogue.

  Written in utilities rather than against a stylesheet: the sizes are the
  design's module, which is Tailwind's default scale — `h-8` is the form tier,
  `h-7` compact, `h-5` a mark — so nothing here needs a class of its own.

  Size and licence sit beside the install button on purpose. Both are decisions
  a person should make before several hundred megabytes move, and a row that
  hides them behind a details view is a row that gets clicked blind.
-->
<script lang="ts">
  import { isFinished } from "$lib/api";
  import type { CatalogModel, Installation } from "$lib/api";

  interface Props {
    /** The model this row describes. */
    model: CatalogModel;
    /** Its installation, when one has been started. */
    install?: Installation;
    /** Start installing. */
    onInstall: (id: string) => void;
    /** Stop installing, keeping the partial file so a retry resumes. */
    onCancel: (id: string) => void;
  }

  let { model, install, onInstall, onCancel }: Props = $props();

  let running = $derived(install !== undefined && !isFinished(install.state));
  let percent = $derived(
    install && install.totalBytes > 0
      ? Math.min(100, (install.progressBytes / install.totalBytes) * 100)
      : 0,
  );

  /** Bytes as a person reads them. */
  function size(bytes: number): string {
    return bytes >= 1_000_000_000
      ? `${(bytes / 1_000_000_000).toFixed(1)} GB`
      : `${Math.round(bytes / 1_000_000)} MB`;
  }
</script>

<div class="border-rule flex items-start gap-4 border-b px-4 py-3 last:border-b-0">
  <div class="min-w-0 flex-1">
    <div class="flex flex-wrap items-center gap-2">
      <span class="text-ink text-sm font-medium">{model.displayName}</span>

      {#if model.resident}
        <span class="border-green text-green-text inline-flex items-center border px-1.5 py-px text-[0.6875rem] leading-4 whitespace-nowrap">
          in memory
        </span>
      {:else if model.installed}
        <span class="border-rule-strong text-ink-dim inline-flex items-center border px-1.5 py-px text-[0.6875rem] leading-4 whitespace-nowrap">
          on disk
        </span>
      {/if}

      {#if model.recommended}
        <span class="border-blue text-blue-text inline-flex items-center border px-1.5 py-px text-[0.6875rem] leading-4 whitespace-nowrap">
          recommended
        </span>
      {/if}
    </div>

    <p class="text-ink-dim mt-1 text-xs leading-5">{model.description}</p>

    <p class="text-ink-faint mt-0.5 font-mono text-xs leading-5 tabular-nums">
      {model.dimensions} dim · {model.contextLength.toLocaleString()} tokens ·
      {size(model.sizeBytes)} · {model.license}
    </p>

    {#if running}
      <!-- Determinate only. A bar that fills from nothing to nothing reads as a
           stall rather than as a start, so it appears once a total is known. -->
      <div class="bg-rule mt-2 h-0.5 w-full">
        <div class="bg-green h-full transition-[width] duration-200" style="width: {percent}%"></div>
      </div>
      <p class="text-ink-faint mt-1 font-mono text-xs tabular-nums">
        {install?.state} · {size(install?.progressBytes ?? 0)} of {size(install?.totalBytes ?? 0)}
      </p>
    {/if}
  </div>

  <div class="flex flex-none items-center gap-3">
    <a
      class="border-rule text-ink-dim hover:border-rule-strong hover:text-ink flex h-7 items-center border px-2.5 text-xs"
      href={model.repositoryUri}
      target="_blank"
      rel="noreferrer"
    >
      Source
    </a>

    {#if running}
      <button
        class="border-rule text-ink-dim hover:border-rule-strong hover:text-ink h-7 border px-2.5 text-xs"
        type="button"
        onclick={() => onCancel(model.id)}
      >
        Cancel
      </button>
    {:else if !model.installed}
      <!-- The button tokens, not a hardcoded white.
           `hover:bg-white` was measured at 1.09:1 in the light theme — a white
           fill under text that stays `--ground` (#f4f5f5), so the label
           vanished on hover. `--btn-bg-hover` is #fff on dark and #000 on
           light, which is what these three tokens exist to get right. -->
      <button
        class="h-7 bg-(--btn-bg) px-2.5 text-xs font-medium text-(--btn-fg) hover:bg-(--btn-bg-hover)"
        type="button"
        onclick={() => onInstall(model.id)}
      >
        Install
      </button>
    {/if}
  </div>
</div>
