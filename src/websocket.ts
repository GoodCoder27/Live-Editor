async function sendHttpPost() {
  const response = await fetch("http://127.0.0", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name: "TypeScript" }),
  });
  const data = await response.json();
  console.log(data.message);
}

const socket = new WebSocket("ws://127.0.0.1:8080/ws");

socket.addEventListener("open", () => {
  console.log("Connected to Rust WebSocket");
  socket.send("Hello via WebSocket!");
});

socket.addEventListener("message", (event) => {
  console.log("Received:", event.data);
});
