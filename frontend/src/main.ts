import "./style.css";
import { createEditor, captureChange, applyChanges, updateRemoteCursor, updateRemoteSelection } from "./editor.ts";
import { initSplits, updateHorizontalSplit } from "./layout.ts";
import { initMobileTabs, ensureDefaultMobilePane } from "./tabs.ts";
import * as monaco from "monaco-editor"

// UserId
const userId = crypto.randomUUID();
let writer: boolean = false;

// 1. Initialize Monaco Editor
let applyingRemoteChange = false;
const container = document.getElementById("editor")!;
const editor = createEditor(container);

const editorPane = document.getElementById('editor')!;
const ioPane = document.getElementById('io-box')!;
const inputBox = document.getElementById("inputBox") as HTMLTextAreaElement;
const outputText = document.getElementById("output") as HTMLDivElement;
inputBox.disabled = true;

function displayOutput(stderr: string, stdout: string) {
    outputText.textContent = stderr + stdout;
}

// 2. Initialize static vertical split
initSplits();

// 3. Initialize mobile tab event listeners
initMobileTabs(editor, editorPane, ioPane);

// 4. Main responsive mode check
const handleScreenResize = () => {
  const isDesktop = window.innerWidth > 768;
  
  updateHorizontalSplit(isDesktop, editor, editorPane, ioPane);
  
  if (!isDesktop) {
    ensureDefaultMobilePane(editorPane, ioPane);
  }
};

// Run on initial load
handleScreenResize();

// 5. Window Resize Listener
window.addEventListener('resize', () => {
  handleScreenResize();
  editor?.layout();
});

// Ensure Monaco renders correctly on first paint
requestAnimationFrame(() => {
  editor?.layout();
});

// show cursors
const remoteCursors = new Map<
    string,
    monaco.editor.IEditorDecorationsCollection
>();
const remoteSelections = new Map<
    string,
    monaco.editor.IEditorDecorationsCollection
>();

// websocket
interface ClientMessage {
    action:
        | "join_room"
        | "code_change"
        | "cursor_change"
        | "run"
        | "input_update"
    data: unknown;
}

// Establish connection
const ws = new WebSocket("ws://127.0.0.1:3000/ws");

ws.onopen = () => {
    console.log("Connected to Rust WebSocket server");

    // Send a typed message serialized to JSON
    const message: ClientMessage = {
        action: "join_room",
        data: JSON.stringify({room_id: "ABC123", user_id: userId,}),
    };

    ws.send(JSON.stringify(message));
};

ws.onmessage = (event: MessageEvent) => {
    try {
        // Parse incoming JSON from Rust
        const response = JSON.parse(event.data);
        if (response.action == "code_update") {
            if (response.user_id == userId) {
                return;
            }
            applyingRemoteChange = true;
            try {
                applyChanges(editor, response.changes);
            } finally {
                applyingRemoteChange = false;
            }
        }
        if (response.action == "input_update") {
            if (response.user_id == userId) return;

            inputBox.value = response.input;
        }
        if (response.action == "cursor_update") {
            if (response.user_id == userId) {
                return;
            }
            if (response.change.selection != null) {
                const selection = response.change.selection;           
                updateRemoteSelection(editor, userId, selection.startLine, selection.startColumn,
                        selection.endLine, selection.endColumn, remoteSelections);
                updateRemoteCursor(editor, userId, response.change.cursor.line, response.change.cursor.column, remoteCursors);
            } else {
                updateRemoteCursor(editor, userId, response.change.cursor.line, response.change.cursor.column, remoteCursors);
                updateRemoteSelection(editor, userId, 0, 0, 0, 0, remoteSelections);
            }
        }
        if (response.action == "output") {
            displayOutput(response.stderr, response.stdout);
        }
        // console.log("Received from server:", response);
        if (response.action == "join_room") {
            if (response.writer == true) {
                editor.updateOptions({
                    readOnly: false
                });
                inputBox.disabled = false;
                writer = true;
                // console.log("writer joined the room");
            }
        }
    } catch (error) {
        console.error("Failed to parse incoming message:", error);
    }
};

ws.onclose = () => {
    console.log("Disconnected from WebSocket server");
};

ws.onerror = (error: Event) => {
    console.error("WebSocket error:", error);
};
// editor change
const model = editor.getModel();
captureChange(model, applyingRemoteChange, (changes) => {
    const message: ClientMessage = {
        action: "code_change",
        data: JSON.stringify(changes), 
    };

    if (applyingRemoteChange)
        return;
        
    if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(message));
    }
});
// input box change
inputBox.addEventListener("input", () => {
    // console.log("Input changed:", inputBox.value);
    if (!writer) return;

    let message: ClientMessage = {
        action: "input_update",
        data: inputBox.value,
    };

    if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(message));
    }
});
// capture selection and cursor change
editor.onDidChangeCursorSelection((event) => {
    const selection = event.selection;
    if (writer) {
        let message: ClientMessage;

        if (selection.isEmpty()) {
            message = {
                action: "cursor_change",
                data: JSON.stringify({
                    cursor: {
                        column: selection.positionColumn,
                        line: selection.positionLineNumber,
                    },
                    selection: null,
                }),
            };
        } else {
            message = {
                action: "cursor_change",
                data: JSON.stringify({
                    cursor: {
                        column: selection.positionColumn,
                        line: selection.positionLineNumber,
                    },
                    selection: {
                        startColumn: selection.startColumn,
                        startLine: selection.startLineNumber,
                        endColumn: selection.endColumn,
                        endLine: selection.endLineNumber,
                    },
                }),
            };
        }
               
        if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(message));
        }
    }
});

const runBtn = document.getElementById("run") as HTMLButtonElement;
runBtn.addEventListener("click", () => {
    if (writer) {
        // console.log("run clicked", event.currentTarget);
        const message: ClientMessage = {
            action: "run",
            data: inputBox.value,
        }

        ws.send(JSON.stringify(message));
    }
});

