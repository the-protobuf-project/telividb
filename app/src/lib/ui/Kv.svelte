<!-- A key and its value on one line, for a dense detail panel. -->
<script lang="ts">
  interface Props {
    /** What it is. */
    label: string;
    /** The value, rendered monospace so figures and ids line up. */
    value: string;
  }

  let { label, value }: Props = $props();
</script>

<div class="kv"><span>{label}</span><span class="mono">{value}</span></div>
