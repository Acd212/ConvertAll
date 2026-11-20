<script lang="ts">
  import type { EngineInfo } from "../types";

  let { engines }: { engines: EngineInfo[] } = $props();
</script>

<div class="flex flex-wrap items-center gap-1.5">
  {#each engines as engine (engine.id)}
    <span
      class="inline-flex items-center gap-1.5 rounded-full border border-line bg-panel px-2.5 py-1 text-[11px] font-medium text-muted"
      title={engine.available
        ? `${engine.version ?? "Disponible"}${engine.path ? `\n${engine.path}` : ""}`
        : "No disponible"}
    >
      <span
        class="h-1.5 w-1.5 shrink-0 rounded-full"
        class:bg-success={engine.available}
        class:bg-danger={!engine.available}
      ></span>
      {engine.name}
    </span>
  {/each}
</div>
