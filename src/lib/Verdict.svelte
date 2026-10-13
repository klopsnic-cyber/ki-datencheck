<script>
  import { Tween } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { ShieldCheck, ShieldAlert, ShieldX } from "@lucide/svelte";
  import { LEVEL, groupColor } from "./api.js";

  let { report } = $props();

  const R = 46, C = 2 * Math.PI * R;
  const score = new Tween(0, { duration: 900, easing: cubicOut });
  $effect(() => { score.target = report.score; });
  let lv = $derived(LEVEL[report.level]);
  const ICON = { gruen: ShieldCheck, gelb: ShieldAlert, rot: ShieldX };
  let Icon = $derived(ICON[report.level]);
</script>

<div class="verdict card" style="--lv:{lv.color}">
  <div class="gauge">
    <svg viewBox="0 0 110 110">
      <circle cx="55" cy="55" r={R} class="track" />
      <circle cx="55" cy="55" r={R} class="bar" stroke-dasharray="{(C * score.current) / 100} {C}" />
    </svg>
    <div class="ic"><Icon size={30} /></div>
  </div>
  <div class="txt">
    <span class="word">{lv.word}</span>
    <h2>{report.headline}</h2>
    {#each report.reasons as r, i}<p style="--d:{i * 80}ms">{r}</p>{/each}
    {#if report.level === "gruen"}<p>Nichts Auffälliges erkannt – trotzdem kurz selbst drüberschauen.</p>{/if}
  </div>
</div>

{#if report.categories.length}
  <div class="chips">
    {#each report.categories as c, i (c.label)}
      <span class="chip" style="--c:{groupColor(c.group)}; --d:{i * 45}ms"><b>{c.count}</b> {c.label}</span>
    {/each}
  </div>
{/if}

<style>
  .verdict {
    display: flex; gap: 20px; align-items: center; padding: 18px 22px; position: relative; overflow: hidden;
    border-color: color-mix(in srgb, var(--lv) 35%, var(--border));
    background: linear-gradient(120deg, color-mix(in srgb, var(--lv) 9%, white), white 60%);
    animation: rise 0.5s var(--ease) backwards; transition: border-color 0.4s, background 0.4s;
  }
  .gauge { position: relative; width: 96px; height: 96px; flex-shrink: 0; }
  .gauge svg { width: 100%; height: 100%; transform: rotate(-90deg); }
  .track { fill: none; stroke: #e9ecef; stroke-width: 9; }
  .bar { fill: none; stroke: var(--lv); stroke-width: 9; stroke-linecap: round; transition: stroke 0.4s; filter: drop-shadow(0 3px 6px color-mix(in srgb, var(--lv) 40%, transparent)); }
  .ic { position: absolute; inset: 0; display: grid; place-items: center; color: var(--lv); animation: pop 0.6s var(--ease) backwards 0.2s; }
  .word { font-size: 11.5px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.09em; color: var(--lv); }
  h2 { font-size: 21px; margin: 2px 0 6px; }
  p { margin: 4px 0 0; color: var(--muted); line-height: 1.45; font-size: 13.5px; animation: rise 0.5s var(--ease) backwards; animation-delay: calc(150ms + var(--d, 0ms)); }
  .chips { display: flex; flex-wrap: wrap; gap: 8px; }
  .chip {
    padding: 5px 11px; border-radius: 999px; font-size: 12.5px; font-weight: 550; color: var(--c);
    background: color-mix(in srgb, var(--c) 10%, white); border: 1px solid color-mix(in srgb, var(--c) 25%, white);
    animation: pop 0.45s var(--ease) backwards; animation-delay: var(--d);
  }
  .chip b { font-weight: 750; }
  @keyframes rise { from { opacity: 0; transform: translateY(8px); } }
  @keyframes pop { from { opacity: 0; transform: scale(0.6); } }
</style>
