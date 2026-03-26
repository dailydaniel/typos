<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { appState } from "./state.svelte";
  import { getDocument, GlobalWorkerOptions } from "pdfjs-dist";
  import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";

  GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

  interface Props {
    onNavigate: (id: string) => void;
  }

  let { onNavigate }: Props = $props();

  let pdfContainer: HTMLDivElement | undefined = $state();

  function hrefToNoteId(href: string): string | null {
    let id = href
      .replace(/^\.\//, "")
      .replace(/^\//, "")
      .replace(/\.typ$/, "")
      .replace(/^notes\//, "")
      .replace(/--/g, "/");
    return id || null;
  }

  // --- HTML mode ---

  const darkPreviewCss = `<style>
    html { background: #262220; color: #e8e2d9; }
    body { background: #262220; color: #e8e2d9; }
    a { color: #c4a882; }
    hr { border-color: #3d3732; }
    code, pre { background: #302b27; color: #e8e2d9; }
    table, th, td { border-color: #3d3732; }
    th { background: #302b27; }
  </style>`;

  const previewHtml = $derived(() => {
    if (!appState.previewHtml || appState.previewFormat !== "html") return "";
    const script = `<script>
      document.addEventListener('click', function(e) {
        var a = e.target.closest('a');
        if (!a) return;
        e.preventDefault();
        var href = a.getAttribute('href') || '';
        window.parent.postMessage({ type: 'note-link', href: href }, '*');
      });
    <\/script>`;
    const dark = appState.darkMode ? darkPreviewCss : "";
    return dark + appState.previewHtml + script;
  });

  function onMessage(e: MessageEvent) {
    if (e.data?.type !== "note-link") return;
    const id = hrefToNoteId(e.data.href);
    if (id) onNavigate(id);
  }

  // --- PDF mode ---

  async function renderPdf(base64: string, container: HTMLDivElement) {
    try {
      container.innerHTML = "";

      const binaryStr = atob(base64);
      const bytes = new Uint8Array(binaryStr.length);
      for (let i = 0; i < binaryStr.length; i++) {
        bytes[i] = binaryStr.charCodeAt(i);
      }

      const doc = await getDocument({ data: bytes }).promise;

      // Wait for layout to compute container width
      let containerWidth = container.clientWidth;
      if (containerWidth === 0) {
        await new Promise<void>((resolve) => requestAnimationFrame(() => {
          containerWidth = container.clientWidth;
          resolve();
        }));
      }
      if (containerWidth === 0) containerWidth = 600; // fallback

      for (let i = 1; i <= doc.numPages; i++) {
        const page = await doc.getPage(i);
        const unscaledViewport = page.getViewport({ scale: 1 });
        // Slight zoom to reduce visible page margins
        const scale = (containerWidth / unscaledViewport.width) * 1.25;
        const viewport = page.getViewport({ scale });
        // Center the page (negative margin crops side margins equally)
        const offsetX = (viewport.width - containerWidth) / 2;

        // Page wrapper
        const pageDiv = document.createElement("div");
        pageDiv.style.position = "relative";
        pageDiv.style.width = `${containerWidth}px`;
        pageDiv.style.height = `${viewport.height}px`;
        pageDiv.style.overflow = "hidden";
        pageDiv.style.marginBottom = "4px";

        // Canvas (retina)
        const canvas = document.createElement("canvas");
        const dpr = window.devicePixelRatio || 1;
        canvas.width = Math.floor(viewport.width * dpr);
        canvas.height = Math.floor(viewport.height * dpr);
        canvas.style.width = `${viewport.width}px`;
        canvas.style.height = `${viewport.height}px`;
        canvas.style.marginLeft = `${-offsetX}px`;
        const ctx = canvas.getContext("2d")!;
        ctx.scale(dpr, dpr);

        await page.render({ canvasContext: ctx, viewport }).promise;
        pageDiv.appendChild(canvas);

        // Clickable links from PDF annotations
        try {
          const annotations = await page.getAnnotations();
          const pageHeight = unscaledViewport.height;
          for (const annot of annotations) {
            // annotationType 2 = LINK; url may be empty for relative paths, use unsafeUrl
            const href = annot.url || annot.unsafeUrl;
            if (annot.annotationType !== 2 || !href) continue;
            // annot.rect = [x1, y1, x2, y2] in PDF coords (origin bottom-left)
            const [x1, y1, x2, y2] = annot.rect;
            const link = document.createElement("a");
            link.href = href;
            link.style.position = "absolute";
            link.style.left = `${x1 * scale - offsetX}px`;
            link.style.top = `${(pageHeight - y2) * scale}px`;
            link.style.width = `${(x2 - x1) * scale}px`;
            link.style.height = `${(y2 - y1) * scale}px`;
            link.style.cursor = "pointer";
            link.onclick = (e) => {
              e.preventDefault();
              const id = hrefToNoteId(href);
              if (id) onNavigate(id);
            };
            pageDiv.appendChild(link);
          }
        } catch (e) {
          console.warn("Annotation extraction failed:", e);
        }

        container.appendChild(pageDiv);
      }
    } catch (e) {
      console.error("PDF render error:", e);
      container.innerHTML = `<div style="color:red;padding:16px">PDF render error: ${e}</div>`;
    }
  }

  $effect(() => {
    if (appState.previewFormat === "pdf" && appState.previewHtml && pdfContainer) {
      renderPdf(appState.previewHtml, pdfContainer);
    }
  });

  onMount(() => {
    window.addEventListener("message", onMessage);
  });

  onDestroy(() => {
    window.removeEventListener("message", onMessage);
  });
</script>

<div class="preview">
  <div class="preview-header">
    <span class="preview-label">Preview</span>
    <button
      class="icon-btn"
      onclick={() => (appState.previewOpen = false)}
      title="Hide preview"
    >×</button>
  </div>
  <div class="preview-body">
    {#if appState.previewLoading}
      <div class="loading">Compiling…</div>
    {:else if appState.previewHtml}
      {#if appState.previewFormat === "html"}
        <iframe
          srcdoc={previewHtml()}
          title="Note preview"
          sandbox="allow-same-origin allow-scripts"
        ></iframe>
      {:else}
        <div class="pdf-container" class:pdf-dark={appState.darkMode} bind:this={pdfContainer}></div>
      {/if}
    {:else}
      <div class="empty">No preview available</div>
    {/if}
  </div>
</div>

<style>
  .preview {
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .preview-label {
    font-size: 12px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .preview-body {
    flex: 1;
    overflow: hidden;
  }
  iframe {
    width: 100%;
    height: 100%;
    border: none;
  }
  .pdf-container {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }
  .pdf-container :global(a) {
    opacity: 0;
    display: block;
  }
  .pdf-container :global(a:hover) {
    opacity: 1;
    background: rgba(255, 255, 0, 0.15);
    border-radius: 2px;
  }
  .pdf-dark :global(canvas) {
    filter: invert(0.88) hue-rotate(180deg);
  }
  .loading, .empty {
    padding: 24px;
    text-align: center;
    color: var(--text-secondary);
    font-size: 13px;
  }
  .icon-btn {
    font-size: 16px;
    padding: 2px 6px;
    border: none;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    line-height: 1;
  }
  .icon-btn:hover {
    color: var(--text);
  }
</style>
