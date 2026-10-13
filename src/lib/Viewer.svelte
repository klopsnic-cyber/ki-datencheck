<script>
  import { save } from "@tauri-apps/plugin-dialog";
  import { Copy, Check, Save, Highlighter, EyeOff } from "@lucide/svelte";
  import { api, groupColor } from "./api.js";

  let { report, name = "Text" } = $props();
  let mode = $state("marked"); // marked | redacted
  let copied = $state(false);

  // Sehr lange Texte nur anzeigen, soweit es flüssig bleibt; Kopieren/Speichern nimmt immer alles.
  const LIMIT = 4000;
  let shown = $derived(report.segments.slice(0, LIMIT));

  async function copy() {
    await navigator.clipboard.writeText(report.redacted);
    copied = true;
    setTimeout(() => (copied = false), 1800);
  }

  async function saveFile() {
    const base = name.replace(/\.[^.]+$/, "");
    const p = await save({ defaultPath: `${base}-anonymisiert.txt`, filters: [{ name: "Text", extensions: ["txt"] }] });
    if (p) await api.saveText(p, report.redacted);
  }
</script>

<div class="viewer card">
  <div class="bar">
    <div class="seg">
      <button class="ghost" class:on={mode === "marked"} onclick={() => (mode = "marked")}><Highlighter size={14} /> Markiert</button>
      <button class="ghost" class:on={mode === "redacted"} onclick={() => (mode = "redacted")}><EyeOff size={14} /> Anonymisiert</button>
    </div>
    <span class="grow"></span>
    <button onclick={copy} class:primary={report.level !== "gruen"}>
      {#if copied}<Check size={14} /> Kopiert{:else}<Copy size={14} /> Anonymisiert kopieren{/if}
    </button>
    <button class="ghost" onclick={saveFile} title="Anonymisierte Fassung speichern"><Save size={15} /></button>
  </div>
  <div class="text">
    {#key mode}
      <div class="fade">
        {#each shown as s, i}
          {#if s.k}
            <mark style="--c:{groupColor(s.group)}; --d:{Math.min(i, 60) * 12}ms" title={s.label} class:red={mode === "redacted"}>{mode === "redacted" ? s.ph : s.t}</mark>
          {:else}{s.t}{/if}
        {/each}
        {#if report.segments.length > LIMIT}<p class="more">… Vorschau gekürzt – Kopieren und Speichern enthalten den ganzen Text.</p>{/if}
      </div>
    {/key}
  </div>
</div>

<style>
  .viewer { flex: 1; min-height: 0; display: flex; flex-direction: column; padding: 10px; }
  .bar { display: flex; gap: 8px; align-items: center; padding-bottom: 10px; border-bottom: 1px solid var(--border); }
  .seg { display: flex; gap: 2px; background: var(--surface-2); padding: 3px; border-radius: 11px; }
  .seg button { padding: 5px 11px; color: var(--muted); border-radius: 8px; }
  .seg button.on { background: white; color: var(--text); box-shadow: var(--shadow); }
  .grow { flex: 1; }
  .bar > button { padding: 6px 12px; font-size: 13px; }
  .text { flex: 1; overflow: auto; padding: 14px 8px 8px; white-space: pre-wrap; word-break: break-word; line-height: 1.7; font-size: 13.5px; user-select: text; -webkit-user-select: text; }
  .fade { animation: fade 0.35s var(--ease); }
  mark {
    color: inherit; padding: 1px 4px; border-radius: 5px; background: color-mix(in srgb, var(--c) 15%, white);
    box-shadow: inset 0 -2px 0 color-mix(in srgb, var(--c) 55%, white); animation: glow 0.6s var(--ease) backwards; animation-delay: var(--d);
  }
  mark.red { color: var(--c); font-weight: 650; font-size: 12px; letter-spacing: 0.02em; }
  .more { color: var(--muted); font-style: italic; }
  @keyframes fade { from { opacity: 0; transform: translateY(4px); } }
  @keyframes glow { from { background: transparent; box-shadow: none; } }
</style>
