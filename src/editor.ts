import * as monaco from "monaco-editor";

export interface CodeChange {
  currentCode: string;
  rangeOffset: number;
  rangeLength: number;
  text: string;
};

export function createEditor(container: HTMLElement) {
  return monaco.editor.create(container, {
    language: "cpp",
    theme: "vs-dark",
    automaticLayout: false,
  });
}

export function captureChange(model: monaco.editor.ITextModel | null, onChange: (change: CodeChange) => void) {
  if (!model) {
    return;
  }
  const changeDisposable = model.onDidChangeContent((event: monaco.editor.IModelContentChangedEvent) => {
      // Get the updated content
      const currentCode = model.getValue();
      const lastChange = event.changes[event.changes.length-1];
      console.log('Editor content changed!', currentCode);
      
      // The 'event' object contains metadata about what exactly changed
      console.log('Changes details:', event.changes);

      if(!lastChange)
        return;
      
      const change: CodeChange = {
        currentCode: currentCode,
        rangeOffset: event.changes[event.changes.length-1].rangeOffset,
        rangeLength: event.changes[event.changes.length-1].rangeLength,
        text: event.changes[event.changes.length-1].text
      };

      onChange(change);
  });
  return changeDisposable;
}
