import * as monaco from "monaco-editor";

export interface CodeChange {
  //currentCode: string;
  range_offset: number;
  range_length: number;
  text: string;
};

// readOnly: true
// editor.updateOptions({
//readOnly: true
// });
// editor-delta-decorations

export function createEditor(container: HTMLElement) {
  return monaco.editor.create(container, {
    language: "cpp",
    theme: "vs-dark",
    automaticLayout: false,
    readOnly: true
  });
}

export function captureChange(model: monaco.editor.ITextModel | null, applyingRemoteChange: boolean, onChange: (change: CodeChange) => void) {
  if (!model || applyingRemoteChange) {
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
        // currentCode: currentCode,
        range_offset: lastChange.rangeOffset,
        range_length: lastChange.rangeLength,
        text: lastChange.text
      };

      onChange(change);
  });
  return changeDisposable;
}

export function applyChange(editor: monaco.editor.IStandaloneCodeEditor, change: CodeChange) {
  const model = editor.getModel()!;

    const start = model.getPositionAt(change.range_offset);
    const end = model.getPositionAt(
        change.range_offset + change.range_length
    );

    model.applyEdits([{
        range: new monaco.Range(
            start.lineNumber,
            start.column,
            end.lineNumber,
            end.column
        ),
        text: change.text,
        forceMoveMarkers: true,
    }]);
}

export function updateRemoteCursor(
    editor: monaco.editor.IStandaloneCodeEditor,
    userId: string,
    lineNumber: number,
    column: number,
    remoteCursors: Map<string, monaco.editor.IEditorDecorationsCollection>
) {
    const model = editor.getModel();
    if (!model) return;

    let cursor = remoteCursors.get(userId);

    if (!cursor) {
        cursor = editor.createDecorationsCollection();
        remoteCursors.set(userId, cursor);
    }

    const position = model.validatePosition({
        lineNumber,
        column,
    });

    cursor.set([{
        range: new monaco.Range(
            position.lineNumber,
            position.column,
            position.lineNumber,
            position.column,
        ),
        options: {
            beforeContentClassName: "remote-cursor",
        },
    }]);
}

export function updateRemoteSelection(
    editor: monaco.editor.IStandaloneCodeEditor,
    userId: string,
    startLine: number,
    startColumn: number,
    endLine: number,
    endColumn: number,
    remoteSelections: Map<string, monaco.editor.IEditorDecorationsCollection>,
){
    const model = editor.getModel();
    if (!model) return;

    let decoration = remoteSelections.get(userId);

    if (!decoration) {
        decoration = editor.createDecorationsCollection();
        remoteSelections.set(userId, decoration);
    }

    const range = new monaco.Range(
        startLine,
        startColumn,
        endLine,
        endColumn,
    );

    decoration.set([{
        range: model.validateRange(range),
        options: {
            className: "remote-selection",
            isWholeLine: false,
        },
    }]);
}
