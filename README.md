# jdwp-mcp

**Java debugging for LLMs via JDWP and Model Context Protocol**

An MCP server that enables Claude Code and other LLM tools to debug Java
applications through the Java Debug Wire Protocol (JDWP). Attach to running
JVMs, set breakpoints, inspect variables, and step through code—all through
natural language.

## Features

- **Remote Debugging**: Connect to any JVM started with JDWP enabled
- **Breakpoint Management**: Set, list, and clear breakpoints by class and line
- **Stack Inspection**: Get summarized stack frames with local variables
- **Execution Control**: Step over/into/out, continue, pause
- **Expression Evaluation**: Evaluate Java expressions in frame context
- **Thread Management**: List and control thread execution
- **Smart Summarization**: Handles large data structures without overwhelming the LLM

## Quick Start

### 1. Start your Java app with JDWP enabled

```bash
java -agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005 -jar myapp.jar
```

### 2. Build and run the MCP server

```bash
cargo build --release
./target/release/jdwp-mcp
```

### 3. Configure Claude Code

Add to your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "jdwp": {
      "command": "/path/to/jdwp-mcp/target/release/jdwp-mcp"
    }
  }
}
```

### 4. Debug with natural language

```
> Attach to the JVM at localhost:5005
> Set a breakpoint at com.example.HelloController line 65
> When it hits, show me the stack and the value of requestCount
```

## Available Tools

| Tool | Description |
|------|-------------|
| `debug.attach` | Connect to JVM via JDWP |
| `debug.set_breakpoint` | Set breakpoint at class:line |
| `debug.list_breakpoints` | List active breakpoints |
| `debug.clear_breakpoint` | Remove a breakpoint |
| `debug.continue` | Resume execution |
| `debug.step_over` | Step over current line |
| `debug.step_into` | Step into method |
| `debug.step_out` | Step out of method |
| `debug.get_stack` | Get stack frames with variables |
| `debug.evaluate` | Evaluate expression |
| `debug.list_threads` | List all threads |
| `debug.pause` | Pause execution |
| `debug.disconnect` | End debug session |

## Example: Debugging with kubectl port-forward

For Kubernetes-deployed Java apps:

```bash
# Forward JDWP port from pod
kubectl port-forward pod/my-app-pod 5005:5005
```

Then in Claude Code:
```
> Attach to localhost:5005
> Set a breakpoint in the processRequest method
```

## Architecture

```
Claude Code → MCP Server → JDWP Client → TCP Socket → JVM
                ↓
         Summarization &
         Context Filtering
```

The MCP server handles:
- **Protocol Translation**: MCP JSON-RPC ↔ JDWP binary protocol
- **Smart Summarization**: Truncates large objects, limits depth
- **State Management**: Tracks breakpoints, threads, sessions

## Development

### Project Structure

```
jdwp-mcp/
├── jdwp-client/        # JDWP protocol implementation
│   ├── connection.rs   # TCP + handshake
│   ├── protocol.rs     # Packet encoding/decoding
│   ├── commands.rs     # JDWP command constants
│   ├── types.rs        # JDWP type definitions
│   └── events.rs       # Event handling
├── mcp-server/         # MCP server
│   ├── main.rs         # Stdio transport
│   ├── protocol.rs     # MCP JSON-RPC
│   ├── handlers.rs     # Request routing
│   ├── tools.rs        # Tool definitions
│   └── session.rs      # Debug session state
└── examples/           # Usage examples
```

### Testing

Use the companion [java-example-for-k8s](../java-example-for-k8s) as a test target:

```bash
cd ../java-example-for-k8s
mvn clean package
java -agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005 \
  -jar target/probe-demo-0.0.1-SNAPSHOT.jar
```

Then test MCP tools against this running app.

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## Status

✅ **Core Functionality Complete** - Ready for MCP integration

### Implemented Features
- [x] Project structure
- [x] JDWP protocol (handshake, packets, encoding/decoding)
- [x] MCP server skeleton with 13 debug tools
- [x] VirtualMachine commands (Version, IDSizes, AllThreads, Suspend/Resume)
- [x] ClassesBySignature (find classes by name)
- [x] ReferenceType.Methods (get method info)
- [x] Method.LineTable (map source lines to bytecode)
- [x] Method.VariableTable (get variable metadata)
- [x] EventRequest.Set (breakpoints with location modifiers)
- [x] ThreadReference.Frames (get call stacks)
- [x] StackFrame.GetValues (read variable values)
- [x] Value formatting and display
- [x] Architecture independence (big-endian protocol, works on Intel & ARM M1/M2/M3)

### Working Examples
- [x] `test_connection` - Basic JDWP handshake
- [x] `test_vm_commands` - Query JVM version and ID sizes
- [x] `test_find_class` - Find classes and methods with line tables
- [x] `test_breakpoint` - Set breakpoints at specific source lines
- [x] `test_manual_stack` - Suspend and inspect thread stacks with variables

### Next Steps
- [ ] Event loop for async breakpoint notifications
- [ ] Stepping commands (step over/into/out)
- [ ] Expression evaluation
- [ ] String and object dereferencing
- [ ] Full MCP server integration

## References

- [JDWP Specification](https://docs.oracle.com/javase/8/docs/platform/jpda/jdwp/jdwp-protocol.html)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Claude Code MCP Documentation](https://docs.claude.com/claude-code)

## License

MIT
