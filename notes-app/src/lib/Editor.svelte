<script lang="ts">
  import { onMount } from "svelte";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, drawSelection } from "@codemirror/view";
  import { EditorState, Compartment } from "@codemirror/state";
  import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
  import { indentUnit, indentService, syntaxHighlighting, defaultHighlightStyle, bracketMatching } from "@codemirror/language";
  import { autocompletion } from "@codemirror/autocomplete";
  import { closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
  import { serverCompletionSource } from "@codemirror/lsp-client";
  import type { LSPClient } from "@codemirror/lsp-client";
  import { createNoteCompletion } from "./noteCompletion";
  import type { NoteMetadata } from "./types";

  interface Props {
    content: string;
    onContentChange: (text: string) => void;
    notes: NoteMetadata[];
    vimMode: boolean;
    lspClient: LSPClient | null;
    fileUri: string;
    onSave?: () => void;
    onClose?: () => void;
  }

  let { content, onContentChange, notes, vimMode, lspClient, fileUri, onSave, onClose }: Props = $props();

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

  /** Toggle // comments for selected lines */
  function toggleTypstComment(view: EditorView): boolean {
    const { state } = view;
    const sel = state.selection.main;
    const fromLine = state.doc.lineAt(sel.from).number;
    const toLine = state.doc.lineAt(sel.to).number;

    // Check if all lines start with //
    let allCommented = true;
    for (let i = fromLine; i <= toLine; i++) {
      if (!state.doc.line(i).text.trimStart().startsWith("//")) {
        allCommented = false;
        break;
      }
    }

    // Build new text for the affected range
    const lines: string[] = [];
    for (let i = fromLine; i <= toLine; i++) {
      const text = state.doc.line(i).text;
      if (allCommented) {
        // Remove "// " or "//" from start (preserving indent)
        lines.push(text.replace(/^(\s*)\/\/ ?/, "$1"));
      } else {
        // Add "// " at column 0
        lines.push("// " + text);
      }
    }

    const rangeFrom = state.doc.line(fromLine).from;
    const rangeTo = state.doc.line(toLine).to;

    try {
      view.dispatch({
        changes: { from: rangeFrom, to: rangeTo, insert: lines.join("\n") },
      });
    } catch {
      // codemirror-lang-typst tree edit error — non-fatal
    }
    return true;
  }

  /** Simple indentation: inherit previous line's indent, +2 after opening brackets */
  const typstIndent = indentService.of((context, pos) => {
    const line = context.lineAt(pos);
    if (line.number === 1) return 0;
    const prevLine = context.lineAt(line.from - 1);
    const prevText = prevLine.text;
    const prevIndent = prevText.match(/^\s*/)?.[0].length ?? 0;
    const trimmed = prevText.trimEnd();
    if (trimmed.endsWith("[") || trimmed.endsWith("(") || trimmed.endsWith("{")) {
      return prevIndent + 2;
    }
    return prevIndent;
  });

  function createExtensions(vimExt: any) {
    const completionSources = [createNoteCompletion(notes)];
    if (lspClient) completionSources.push(serverCompletionSource);

    return [
      vimCompartment.of(vimExt),
      drawSelection(),
      lineNumbers(),
      highlightActiveLine(),
      highlightActiveLineGutter(),
      history(),
      bracketMatching(),
      closeBrackets(),
      syntaxHighlighting(defaultHighlightStyle),
      autocompletion({
        override: completionSources,
        activateOnTyping: true,
      }),
      ...(lspClient ? [
        lspClient.plugin(fileUri, "typst"),
      ] : []),
      indentUnit.of("  "),
      typstIndent,
      EditorView.domEventHandlers({
        keydown(event, view) {
          if ((event.metaKey || event.ctrlKey) && event.key === "/") {
            event.preventDefault();
            toggleTypstComment(view);
            return true;
          }
        },
      }),
      keymap.of([indentWithTab, ...closeBracketsKeymap, ...defaultKeymap, ...historyKeymap]),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          skipNextExternal = true;
          onContentChange(update.state.doc.toString());
        }
      }),
      EditorView.theme({
        "&": { height: "100%", fontSize: "14px" },
        ".cm-scroller": { overflow: "auto", fontFamily: "var(--font-mono)" },
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
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    color: var(--text-secondary);
  }
  /* Selection */
  .editor-container :global(.cm-selectionBackground) {
    background: rgba(92, 74, 58, 0.18) !important;
  }
  .editor-container :global(.cm-editor.cm-focused .cm-selectionBackground) {
    background: rgba(92, 74, 58, 0.22) !important;
  }
  .editor-container :global(.cm-content ::selection) {
    background: rgba(92, 74, 58, 0.22);
  }
  /* Vim cursor in normal mode */
  .editor-container :global(.cm-fat-cursor) {
    background: rgba(44, 40, 37, 0.7) !important;
    color: var(--surface) !important;
  }
</style>
