import type { CompletionContext, CompletionResult } from "@codemirror/autocomplete";
import type { NoteMetadata } from "./types";

/**
 * Creates a CodeMirror completion source for note references.
 * Triggers on:
 *   - #xlink("...  → suggests note IDs
 *   - "@...         → suggests note IDs (inside property strings)
 */
export function createNoteCompletion(notes: NoteMetadata[]) {
  return function noteCompletion(context: CompletionContext): CompletionResult | null {
    // Match #xlink("partial-id
    const xlinkMatch = context.matchBefore(/#xlink\(\s*"[^"]*/);
    if (xlinkMatch) {
      const quoteIdx = xlinkMatch.text.lastIndexOf('"');
      const from = xlinkMatch.from + quoteIdx + 1;
      return {
        from,
        options: notes.map((n) => ({
          label: n.id,
          detail: `${n.title} (${n.type})`,
          type: "variable",
        })),
        validFor: /^[a-zA-Z0-9/_-]*$/,
      };
    }

    // Match "@partial-id inside property values
    const atMatch = context.matchBefore(/"@[^"]*/);
    if (atMatch) {
      const from = atMatch.from + 2; // after "@
      return {
        from,
        options: notes.map((n) => ({
          label: n.id,
          detail: `${n.title} (${n.type})`,
          type: "variable",
        })),
        validFor: /^[a-zA-Z0-9/_-]*$/,
      };
    }

    return null;
  };
}
