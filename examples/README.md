# JDWP MCP Examples

This directory contains examples of using the JDWP MCP debugger with Claude Code for real-world debugging scenarios.

## Running the Examples

All examples assume:
1. The JDWP MCP server is built and configured
2. Your Java application is running with JDWP enabled on port 5005
3. You're using Claude Code with the MCP server enabled

## Examples

### [observability-debugging.md](observability-debugging.md)
**Real-World Spring Boot Debugging** *(Advanced)*

Investigates an ObservationRegistry post-processing issue similar to [spring-security#15658](https://github.com/spring-projects/spring-security/issues/15658).

**What You'll Learn:**
- Setting breakpoints in framework code (Spring, Micrometer)
- Inspecting Spring application context during initialization
- Verifying bean post-processing
- Debugging missing metrics issues
- Using natural language to navigate complex codebases

**Key Commands:**
```
> Attach to localhost:5005
> Set a breakpoint at AbstractApplicationContext line 869
> Show me the beanFactory variables
> Evaluate this.getBeanNamesForType(ObservationRegistry.class)
```

### Basic Debugging (test_*.rs files)

The Rust test files in this directory demonstrate low-level JDWP protocol usage:

- **test_connection.rs** - Basic JDWP handshake and connection
- **test_vm_commands.rs** - VirtualMachine commands (Version, IDSizes)
- **test_find_class.rs** - Finding classes and methods
- **test_breakpoint.rs** - Setting breakpoints
- **test_manual_stack.rs** - Stack inspection with variables

These are primarily for library development and testing.

## Quick Reference

### Essential Prompts

**Connection:**
```
Attach to the JVM at localhost:5005
```

**Breakpoints:**
```
Set a breakpoint at com.example.MyClass line 42
List all breakpoints
Clear breakpoint bp_1
```

**Execution Control:**
```
Continue execution
Pause all threads
Step over this line
Step into this method
```

**Inspection:**
```
Show me the current stack with variables
List all threads
Get the stack for thread 5
```

**Evaluation:**
```
Evaluate myVariable.toString() in the current frame
```

**Cleanup:**
```
Clear all breakpoints
Disconnect from the debug session
```

## Tips for Effective Debugging

### 1. Strategic Breakpoint Placement

Instead of stepping through every line, set breakpoints at key decision points:
```
> Set a breakpoint where the error condition is checked
> Set a breakpoint at the entry of the suspicious method
```

### 2. Use Stack Inspection

Get the full context when a breakpoint hits:
```
> When the breakpoint hits, show me the full stack with all variables
```

### 3. Expression Evaluation

Inspect complex state without modifying code:
```
> Evaluate myObject.getInternalState() to see what's really happening
```

### 4. Thread Management

For multithreaded issues:
```
> List all threads and show which ones are suspended
> Get the stack for thread 12 to see what it's waiting on
```

## Common Scenarios

### Debugging Spring Boot Applications

```
> Set a breakpoint in AbstractApplicationContext.refresh
> When it hits, evaluate getBeanDefinitionNames() to see all beans
```

### Debugging HTTP Requests

```
> Set a breakpoint at MyController.handleRequest line 45
> Make a curl request to trigger it
> Show me the request parameters and headers
```

### Debugging Async/Reactive Code

```
> List all threads
> Find threads with "reactor-http" in the name
> Get stacks for those threads
```

### Debugging Database Queries

```
> Set a breakpoint at MyRepository.findByUsername
> When it hits, evaluate username to see what's being queried
```

## Remote Debugging

For Kubernetes-deployed applications:

```bash
# Terminal 1: Port forward
kubectl port-forward pod/my-app-pod 5005:5005

# Terminal 2: Claude Code
> Attach to localhost:5005
> Set a breakpoint in production code
```

## Troubleshooting

### Connection Refused
Ensure your Java app is running with:
```bash
-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005
```

### Breakpoint Not Hit
- Verify the class is loaded: `List all threads`
- Check the line number matches your source code
- Ensure the method is actually being called

### Variables Show as "Cannot inspect"
- Thread must be suspended (at a breakpoint or manually paused)
- Try: `Pause all threads` first

## Contributing Examples

Have a good debugging scenario? Add it to this directory:

1. Create a `.md` file describing the scenario
2. Include the problem, debugging steps, and solution
3. Show actual prompts and responses
4. Submit a PR!

## Resources

- [JDWP Specification](https://docs.oracle.com/javase/8/docs/technotes/guides/jpda/jdwp-spec.html)
- [Spring Boot Documentation](https://docs.spring.io/spring-boot/docs/current/reference/html/)
- [Micrometer Documentation](https://micrometer.io/docs)
