<script lang="ts">
  import { onMount } from "svelte";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from "@codemirror/view";
  import { EditorState, Compartment } from "@codemirror/state";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from "@codemirror/language";
  import { autocompletion } from "@codemirror/autocomplete";
  import { createNoteCompletion } from "./noteCompletion";
  import type { NoteMetadata } from "./types";

  interface Props {
    content: string;
    onContentChange: (text: string) => void;
    notes: NoteMetadata[];
    vimMode: boolean;
    onSave?: () => void;
    onClose?: () => void;
  }

  let { content, onContentChange, notes, vimMode, onSave, onClose }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | undefined;
  let skipNextExternal = false;
  const vimCompartment = new Compartment();

  async function loadVimExtension(withCommands: boolean) {
    if (!vimMode) return [];
    try {
      const { vim, Vim } = await import("@replit/codemirror-vim");

      if (withCommands) {
        Vim.defineEx("write", "w", () => { onSave?.(); });
        Vim.defineEx("quit", "q", () => { onClose?.(); });
        Vim.defineEx("wquit", "wq", () => { onSave?.(); onClose?.(); });
      }

      return vim({ status: true });
    } catch {
      return [];
    }
  }

  function createExtensions(vimExt: any) {
    return [
      vimCompartment.of(vimExt),
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
      return [];
    }
  }

  onMount(() => {
    Promise.all([loadTypstLanguage(), loadVimExtension(true)]).then(([langExt, vimExt]) => {
      const extensions = [
        ...createExtensions(vimExt),
        ...(Array.isArray(langExt) ? langExt : [langExt]),
      ];
      view = new EditorView({
        state: EditorState.create({
          doc: content,
          extensions,
        }),
        parent: container,
      });
      view.focus();
    });

    return () => {
      view?.destroy();
    };
  });

  $effect(() => {
    const isVim = vimMode;
    if (!view) return;
    import("@replit/codemirror-vim").then(({ vim, Vim }) => {
      // Re-register commands on toggle
      Vim.defineEx("write", "w", () => { onSave?.(); });
      Vim.defineEx("quit", "q", () => { onClose?.(); });
      Vim.defineEx("wquit", "wq", () => { onSave?.(); onClose?.(); });

      view!.dispatch({
        effects: vimCompartment.reconfigure(isVim ? vim({ status: true }) : []),
      });
    }).catch(() => {});
  });

  $effect(() => {
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
  /* Vim status bar */
  .editor-container :global(.cm-vim-panel) {
    padding: 2px 8px;
    font-family: 'SF Mono', Menlo, Monaco, monospace;
    font-size: 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    color: var(--text-secondary);
  }
  /* Visual mode selection */
  .editor-container :global(.cm-selectionBackground) {
    background: rgba(37, 99, 235, 0.25) !important;
  }
  .editor-container :global(.cm-editor.cm-focused .cm-selectionBackground) {
    background: rgba(37, 99, 235, 0.3) !important;
  }
  /* Vim cursor in normal mode */
  .editor-container :global(.cm-fat-cursor) {
    background: rgba(0, 0, 0, 0.7) !important;
    color: white !important;
  }
</style>
