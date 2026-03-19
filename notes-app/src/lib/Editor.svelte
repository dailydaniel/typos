<script lang="ts">
  import { onMount } from "svelte";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from "@codemirror/language";
  import { autocompletion } from "@codemirror/autocomplete";
  import { createNoteCompletion } from "./noteCompletion";
  import type { NoteMetadata } from "./types";

  interface Props {
    content: string;
    onContentChange: (text: string) => void;
    notes: NoteMetadata[];
  }

  let { content, onContentChange, notes }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | undefined;
  let skipNextExternal = false;

  function createExtensions() {
    return [
      lineNumbers(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      history(),
      bracketMatching(),
      syntaxHighlighting(defaultHighlightStyle),
      autocompletion({
        override: [createNoteCompletion(notes)],
        activateOnTyping: true,
      }),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          skipNextExternal = true;
          onContentChange(update.state.doc.toString());
        }
      }),
      EditorView.theme({
        "&": { height: "100%", fontSize: "14px" },
        ".cm-scroller": { overflow: "auto", fontFamily: "'SF Mono', Menlo, Monaco, monospace" },
        ".cm-content": { padding: "8px 0" },
        ".cm-gutters": { borderRight: "1px solid var(--border)", background: "var(--bg-secondary)" },
      }),
    ];
  }

  async function loadTypstLanguage() {
    try {
      const { typst } = await import("codemirror-lang-typst");
      return typst();
    } catch {
      // Fallback: no language support if typst WASM fails
      return [];
    }
  }

  onMount(() => {
    loadTypstLanguage().then((langExt) => {
      const extensions = [...createExtensions(), ...(Array.isArray(langExt) ? langExt : [langExt])];
      view = new EditorView({
        state: EditorState.create({
          doc: content,
          extensions,
        }),
        parent: container,
      });
    });

    return () => {
      view?.destroy();
    };
  });

  $effect(() => {
    // Track content prop
    const newContent = content;
    if (!view) return;
    if (skipNextExternal) {
      skipNextExternal = false;
      return;
    }
    const current = view.state.doc.toString();
    if (current !== newContent) {
      view.dispatch({
        changes: { from: 0, to: current.length, insert: newContent },
      });
    }
  });
</script>

<div class="editor-container" bind:this={container}></div>

<style>
  .editor-container {
    flex: 1;
    overflow: hidden;
  }
  .editor-container :global(.cm-editor) {
    height: 100%;
  }
</style>
