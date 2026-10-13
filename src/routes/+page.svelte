<script>
  import "$lib/theme.css";
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getVersion } from "@tauri-apps/api/app";
  import { open } from "@tauri-apps/plugin-dialog";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { Lock, Download, X, FileUp, FileText, Type, Sparkles, LoaderCircle, TriangleAlert, Trash2 } from "@lucide/svelte";
  import Verdict from "$lib/Verdict.svelte";
  import Viewer from "$lib/Viewer.svelte";
  import { api, LEVEL } from "$lib/api.js";

  const EXAMPLE = `Hallo, bitte fasse den Arztbrief zusammen:

Patient: Müller, Hans, geb. 03.04.1961
Versichertennr.: A123456780
Anschrift: Hauptstraße 12, 12345 Berlin, Tel. 030 1234567

Diagnose: Diabetes mellitus Typ 2 (E11.9), arterielle Hypertonie.
Medikation: Metformin 1000 mg 1-0-1.

Rückfragen gern an Frau Dr. Schneider (schneider@praxis-beispiel.de).
Login fürs Portal: Passwort: Sommer2024!`;

  let tab = $state("text"); // text | files
  let text = $state("");
  let textReport = $state(null);
  let files = $state([]); // { path, name, busy, report, error }
  let selected = $state(null); // index in files, oder null = Text
  let dragging = $state(false);
  let version = $state("");
  let update = $state(null);
  let updating = $state(false);

  let current = $derived(tab === "files" && selected != null ? files[selected] : null);
  let report = $derived(tab === "files" ? current?.report : textReport);

  let timer;
  $effect(() => {
    const t = text;
    clearTimeout(timer);
    if (!t.trim()) return (textReport = null);
    timer = setTimeout(async () => (textReport = await api.analyzeText(t)), 220);
  });

  onMount(() => {
    const log = (m) => invoke("frontend_log", { msg: String(m) }).catch(() => {});
    window.addEventListener("error", (e) => log(`${e.message} @ ${e.filename}:${e.lineno}`));
    window.addEventListener("unhandledrejection", (e) => log(`Promise: ${e.reason?.stack || e.reason}`));
    getVersion().then((v) => (version = v));
    checkUpdate();
    const iv = setInterval(() => !update && checkUpdate(), 4 * 3600 * 1000);
    invoke("dev_demo").then((d) => d && (text = EXAMPLE));
    const un = getCurrentWebview().onDragDropEvent((e) => {
      const t = e.payload.type;
      if (t === "enter" || t === "over") dragging = true;
      else if (t === "leave") dragging = false;
      else if (t === "drop") {
        dragging = false;
        addFiles(e.payload.paths);
      }
    });
    return () => {
      clearInterval(iv);
      un.then((f) => f());
    };
  });

  async function checkUpdate() {
    try {
      update = await check();
    } catch {} // offline oder noch kein Release
  }

  async function installUpdate() {
    updating = true;
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch {
      updating = false;
    }
  }

  async function pick() {
    const p = await open({ multiple: true, title: "Dateien prüfen" });
    if (p) addFiles(Array.isArray(p) ? p : [p]);
  }

  function addFiles(paths) {
    if (!paths?.length) return;
    tab = "files";
    const start = files.length;
    files = [...files, ...paths.map((path) => ({ path, name: path.split(/[\\/]/).pop(), busy: true, report: null, error: "" }))];
    selected = start;
    paths.forEach(async (path, i) => {
      const f = files[start + i];
      try {
        f.report = await api.analyzeFile(path);
      } catch (e) {
        f.error = String(e);
      }
      f.busy = false;
    });
  }
</script>

<div class="blobs" aria-hidden="true"><i></i><i></i><i></i></div>

<div class="app">
  <header data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <img class="logo" src="/logo.png" alt="" />
      <b data-tauri-drag-region>KI-Datencheck</b>
      {#if version}<small class="ver">{version}</small>{/if}
    </div>
    <span class="local" title="Texte und Dateien werden nur auf diesem Rechner geprüft"><Lock size={13} /> Prüfung nur lokal – nichts verlässt den Rechner</span>
  </header>

  {#if update}
    <div class="update">
      <Download size={15} /> Version {update.version} ist da.
      <button class="primary small" disabled={updating} onclick={installUpdate}>{updating ? "Wird installiert …" : "Jetzt aktualisieren"}</button>
      <button class="ghost small" onclick={() => (update = null)} aria-label="Später"><X size={14} /></button>
    </div>
  {/if}

  <main>
    <section class="input card">
      <div class="tabs">
        <button class="ghost" class:on={tab === "text"} onclick={() => (tab = "text")}><Type size={15} /> Text</button>
        <button class="ghost" class:on={tab === "files"} onclick={() => (tab = "files")}>
          <FileText size={15} /> Dateien {#if files.length}<span class="count">{files.length}</span>{/if}
        </button>
      </div>

      {#if tab === "text"}
        <textarea bind:value={text} placeholder="Text hier einfügen, den du einer KI geben willst …&#10;&#10;Die Prüfung läuft beim Tippen."></textarea>
        <div class="row">
          <button class="ghost small" onclick={() => (text = EXAMPLE)}><Sparkles size={14} /> Beispiel laden</button>
          {#if text}<button class="ghost small" onclick={() => (text = "")}><Trash2 size={14} /> Leeren</button>{/if}
        </div>
      {:else}
        <button class="drop" onclick={pick}>
          <FileUp size={26} />
          <b>Dateien hierher ziehen</b>
          <span class="muted">oder klicken zum Auswählen · PDF, Word, Excel, PowerPoint, Text, CSV, E-Mail …</span>
        </button>
        <ul class="files">
          {#each files as f, i (f.path + i)}
            <li in:fly={{ y: 10, duration: 350 }}>
              <button class="file" class:on={selected === i} onclick={() => (selected = i)}>
                {#if f.busy}<LoaderCircle size={16} class="spin" />
                {:else if f.error}<TriangleAlert size={16} color="var(--gelb)" />
                {:else}<span class="dot" style:background={LEVEL[f.report.level].color}></span>{/if}
                <span class="fname"><b>{f.name}</b><small class="muted">{f.busy ? "wird geprüft …" : f.error || f.report.headline}</small></span>
              </button>
            </li>
          {/each}
        </ul>
        {#if files.length}<button class="ghost small clear" onclick={() => ((files = []), (selected = null))}><Trash2 size={14} /> Liste leeren</button>{/if}
      {/if}
    </section>

    <section class="out">
      {#if current?.error}
        <div class="card empty"><TriangleAlert size={34} color="var(--gelb)" /><h3>{current.name}</h3><p class="muted">{current.error}</p></div>
      {:else if report}
        {#key current?.path ?? "text"}
          <div class="stack" in:fly={{ y: 14, duration: 400 }}>
            <Verdict {report} />
            <Viewer {report} name={current?.name ?? "Text"} />
          </div>
        {/key}
      {:else}
        <div class="card empty">
          <div class="pulse"><img src="/logo.png" alt="" /></div>
          <h3>Erst prüfen, dann in die KI</h3>
          <p class="muted">Füge links Text ein oder zieh Dateien ins Fenster. Du siehst sofort, ob Namen, Gesundheitsdaten, Kennungen oder Passwörter drinstecken – und bekommst eine anonymisierte Fassung zum Kopieren.</p>
        </div>
      {/if}
      <p class="note muted">Automatische Prüfung nach Regeln – sie findet viel, aber nicht alles. Im Zweifel nicht hochladen.</p>
    </section>
  </main>

  {#if dragging}<div class="dropfx"><div><FileUp size={44} /><b>Loslassen zum Prüfen</b></div></div>{/if}
</div>

<style>
  .app { position: relative; height: 100vh; display: flex; flex-direction: column; }
  header { height: 52px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; padding: 0 18px 0 88px; }
  .brand { display: flex; align-items: center; gap: 9px; font-size: 14.5px; }
  .logo { width: 24px; height: 24px; border-radius: 6px; }
  .ver { color: var(--muted); font-size: 11.5px; font-weight: 500; }
  .local { display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 550; color: var(--gruen); background: color-mix(in srgb, var(--gruen) 9%, white); border: 1px solid color-mix(in srgb, var(--gruen) 22%, white); padding: 4px 10px; border-radius: 999px; }
  .update { margin: 0 24px 12px; padding: 8px 12px; display: flex; align-items: center; gap: 10px; justify-content: center; font-size: 13px; border-radius: 12px; background: #e3f7f1; border: 1px solid #b4e6d6; color: #0b6b55; animation: drop 0.5s var(--ease) both; }
  .small { padding: 4px 11px; font-size: 12.5px; }

  main { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(340px, 0.85fr) minmax(0, 1.4fr); gap: 16px; padding: 0 24px 18px; }
  .input { display: flex; flex-direction: column; padding: 10px; min-height: 0; animation: rise 0.5s var(--ease) backwards; }
  .tabs { display: flex; gap: 4px; padding-bottom: 10px; }
  .tabs button { color: var(--muted); padding: 6px 12px; }
  .tabs button.on { color: var(--accent); background: color-mix(in srgb, var(--accent) 10%, white); }
  .count { background: var(--accent); color: white; border-radius: 999px; font-size: 11px; padding: 0 7px; }
  textarea { flex: 1; resize: none; border: 1px solid var(--border); border-radius: 12px; padding: 14px; line-height: 1.6; background: var(--surface-2); outline: none; transition: border-color 0.2s, background 0.2s, box-shadow 0.2s; }
  textarea:focus { border-color: color-mix(in srgb, var(--accent) 50%, var(--border)); background: white; box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent) 12%, transparent); }
  .row { display: flex; gap: 6px; padding-top: 8px; }

  .drop { flex-direction: column; gap: 6px; padding: 26px 16px; border: 2px dashed color-mix(in srgb, var(--accent) 35%, var(--border)); border-radius: 14px; background: color-mix(in srgb, var(--accent) 4%, white); color: var(--accent); text-align: center; }
  .drop:hover { background: color-mix(in srgb, var(--accent) 8%, white); transform: translateY(-1px); }
  .drop span { font-size: 12px; font-weight: 450; }
  .files { list-style: none; margin: 10px 0 0; padding: 0; overflow: auto; flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .file { width: 100%; justify-content: flex-start; text-align: left; gap: 10px; border-color: transparent; background: transparent; padding: 8px 10px; }
  .file:hover { background: var(--surface-2); box-shadow: none; }
  .file.on { background: color-mix(in srgb, var(--accent) 9%, white); border-color: color-mix(in srgb, var(--accent) 25%, white); }
  .dot { width: 11px; height: 11px; border-radius: 50%; flex-shrink: 0; box-shadow: 0 0 0 3px color-mix(in srgb, currentColor 0%, white); }
  .fname { display: flex; flex-direction: column; min-width: 0; }
  .fname b, .fname small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .fname b { font-weight: 550; }
  .fname small { font-weight: 450; font-size: 11.5px; }
  .clear { align-self: flex-start; margin-top: 6px; }
  :global(.spin) { animation: spin 0.9s linear infinite; color: var(--accent); flex-shrink: 0; }

  .out { display: flex; flex-direction: column; gap: 10px; min-height: 0; }
  .stack { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 12px; }
  .empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; gap: 10px; padding: 30px 60px; animation: rise 0.5s var(--ease) backwards 0.1s; }
  .empty p { max-width: 460px; line-height: 1.55; margin: 0; }
  .empty h3 { font-size: 20px; }
  .pulse { width: 92px; height: 92px; border-radius: 26px; display: grid; place-items: center; animation: float 3.2s ease-in-out infinite; }
  .pulse img { width: 92px; height: 92px; border-radius: 22px; box-shadow: 0 14px 34px rgba(15, 159, 132, 0.25); }
  .note { font-size: 11.5px; margin: 0; text-align: center; }

  .dropfx { position: fixed; inset: 0; display: grid; place-items: center; background: rgba(244, 248, 247, 0.75); backdrop-filter: blur(6px); animation: fadein 0.2s; z-index: 10; }
  .dropfx div { display: flex; flex-direction: column; align-items: center; gap: 10px; color: var(--accent); padding: 40px 70px; border: 3px dashed var(--accent); border-radius: 24px; background: white; animation: pop 0.35s var(--ease); }

  .blobs { position: fixed; inset: 0; z-index: -1; overflow: hidden; filter: blur(70px); opacity: 0.55; }
  .blobs i { position: absolute; width: 460px; height: 460px; border-radius: 50%; animation: drift 26s ease-in-out infinite alternate; }
  .blobs i:nth-child(1) { background: #c9f0e3; top: -160px; left: -120px; }
  .blobs i:nth-child(2) { background: #dcd6ff; bottom: -200px; right: -100px; animation-delay: -9s; }
  .blobs i:nth-child(3) { background: #d3eefa; top: 40%; left: 45%; width: 340px; height: 340px; animation-delay: -17s; }
  @keyframes drift { to { transform: translate(90px, 60px) scale(1.15); } }
  @keyframes rise { from { opacity: 0; transform: translateY(12px); } }
  @keyframes drop { from { opacity: 0; transform: translateY(-8px); } }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes float { 50% { transform: translateY(-8px); } }
  @keyframes fadein { from { opacity: 0; } }
  @keyframes pop { from { opacity: 0; transform: scale(0.9); } }
  @media (prefers-reduced-motion: reduce) { .blobs i, .pulse { animation: none; } }
</style>
