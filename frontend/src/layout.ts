import Split from "split.js";
import type * as monaco from "monaco-editor";

let horizontalSplit: any = null;
let verticalSplit: any = null;

export function initSplits(editor: monaco.editor.IStandaloneCodeEditor | null) {
  // Initialize vertical split globally (runs on both mobile and desktop)
  if (!verticalSplit) {
    verticalSplit = Split(['#input', '#output'], {
      sizes: [45, 55],
      minSize: 100,
      gutterSize: 10,
      direction: "vertical"
    });
  }
}

export function updateHorizontalSplit(
  isDesktop: boolean,
  editor: monaco.editor.IStandaloneCodeEditor | null,
  editorPane: HTMLElement,
  ioPane: HTMLElement
) {
  if (isDesktop) {
    // Desktop mode: clear mobile visibility classes
    editorPane.classList.remove('active-pane');
    ioPane.classList.remove('active-pane');

    if (!horizontalSplit) {
      horizontalSplit = Split(['#editor', '.io-box'], {
        sizes: [50, 50],
        minSize: 200,
        gutterSize: 10,
        direction: "horizontal",
        onDrag: () => editor?.layout()
      });
    }
  } else {
    // Mobile mode: destroy horizontal split and clear inline widths
    if (horizontalSplit) {
      horizontalSplit.destroy();
      horizontalSplit = null;
      editorPane.style.width = '';
      ioPane.style.width = '';
    }
  }
}
