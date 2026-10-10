| HOME - FILE - RUN CODE - GLAZE MYSELF |                                      |
|---------------------------------------|--------------------------------------|
| 									                    |                                      |
| 									                    |                                      |
| 								                      |              Input Area              |
| 									                    |                                      |
| 									                    |                                      |
| 				Editor Area                   |--------------------------------------|
| 									                    |                                      |
| 									                    |                                      |
| 									                    |      Output/STDERR area              |
| 									                    |                                      |
| 									                    |                                      |
| 									                    |                                      |
|---------------------------------------|--------------------------------------|


### What is websocket used for?
	1. Running code
	2. Sharing code with others

### What should we send
```rust
	#[derive(Serialize, Deserialize, Debug)]
	struct ClientMessage {
		action: String,
		text: String,
		rangeOffset: u32,
		rangeLength: u32,
	};
```
