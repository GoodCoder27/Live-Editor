import "./style.css";
import { createEditor, captureChange, applyChange, updateRemoteCursor, updateRemoteSelection } from "./editor.ts";
import { initSplits, updateHorizontalSplit } from "./layout.ts";
import { initMobileTabs, ensureDefaultMobilePane } from "./tabs.ts";
import * as monaco from "monaco-editor"

// UserId
const userId = crypto.randomUUID();

// 1. Initialize Monaco Editor
let applyingRemoteChange = false;
const container = document.getElementById("editor")!;
const editor = createEditor(container);

const editorPane = document.getElementById('editor')!;
const ioPane = document.getElementById('io-box')!;

// 2. Initialize static vertical split
initSplits(editor);

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
editor.setValue("Hello world!");
updateRemoteCursor(editor, userId, 1, 3, remoteCursors);
updateRemoteSelection(editor, userId, 1, 1, 1, 3, remoteSelections)

// websocket
interface ClientMessage {
    action:
        | "join_room"
        | "code_change"
        | "cursor_change"
        | "user_joined"
        | "user_left"
        | "run"
        | "output";
    data: unknown;
}

interface ServerMessage {
    status: string;
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
                applyChange(editor, response.change);
            } finally {
                applyingRemoteChange = false;
            }
        }
        console.log("Received from server:", response);
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

const model = editor.getModel();
captureChange(model, applyingRemoteChange, (change) => {
    const message: ClientMessage = {
        action: "code_change",
        data: JSON.stringify(change), 
    };

    if (applyingRemoteChange)
        return;
        
    if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(message));
    }
});

const runBtn = document.getElementById("run") as HTMLButtonElement;
runBtn.addEventListener("click", (event: MouseEvent) => {
    console.log("run clicked", event.currentTarget);

    const message: ClientMessage = {
        action: "run",
        data: model!.getValue(),
    }

    ws.send(JSON.stringify(message));
});

