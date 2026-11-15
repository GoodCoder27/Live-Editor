// src/tabs.ts
import type * as monaco from "monaco-editor";

export function initMobileTabs(
  editor: monaco.editor.IStandaloneCodeEditor | null,
  editorPane: HTMLElement,
  ioPane: HTMLElement
) {
  const mobileTabs = document.querySelectorAll<HTMLButtonElement>('.mobile-tab');

  mobileTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      mobileTabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      const target = tab.getAttribute('data-target');

      if (target === 'editor') {
        editorPane.classList.add('active-pane');
        ioPane.classList.remove('active-pane');
        editor?.layout(); // Refresh Monaco layout when switching back to editor
      } else if (target === 'io-box') {
        editorPane.classList.remove('active-pane');
        ioPane.classList.add('active-pane');
      }
    });
  });
}

export function ensureDefaultMobilePane(
  editorPane: HTMLElement,
  ioPane: HTMLElement
) {
  const mobileTabs = document.querySelectorAll<HTMLButtonElement>('.mobile-tab');
  
  // If neither pane has an active class yet, default to Editor view
  if (!editorPane.classList.contains('active-pane') && !ioPane.classList.contains('active-pane')) {
    editorPane.classList.add('active-pane');
    
    mobileTabs.forEach(t => {
      if (t.getAttribute('data-target') === 'editor') {
        t.classList.add('active');
      } else {
        t.classList.remove('active');
      }
    });
  }
}
