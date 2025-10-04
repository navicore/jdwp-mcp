# Real-World Example: Debugging Spring Boot ObservationRegistry Post-Processing

## Background

This example demonstrates using the JDWP MCP debugger to investigate a real-world Spring Boot observability issue similar to [spring-security#15658](https://github.com/spring-projects/spring-security/issues/15658).

**The Problem**: ObservationRegistry not being properly post-processed by Spring, leading to missing metrics (like `http.server.requests`).

**The Goal**: Use natural language debugging with Claude Code to verify that ObservationRegistry is correctly initialized and post-processed.

## Prerequisites

1. Java application running with JDWP enabled:
   ```bash
   java -agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005 -jar myapp.jar
   ```

2. JDWP MCP server configured in Claude Code:
   ```bash
   claude mcp add --scope project jdwp /path/to/jdwp-mcp/target/release/jdwp-mcp
   ```

## Debugging Session

### Prerequisites Check

First, verify if metrics are working (this tells us if ObservationRegistry is healthy):

```bash
# Check if http.server.requests metric exists
curl http://localhost:8081/actuator/metrics/http.server.requests
```

If you get a 404 or "No such metric" error, you have the bug. If you get data, post-processing worked.

### Step 1: Attach to the JVM

**Prompt:**
```
Attach to the JVM at localhost:5005
```

**Response:**
```
Connected to JVM at localhost:5005 (session: session_xyz)
```

### Step 2: Set Breakpoint in Application Startup

We want to catch the application during initialization to inspect the ObservationRegistry bean.

**Prompt:**
```
Set a breakpoint at com.example.probedemo.ProbeDemoApplication line 13
where the main method calls SpringApplication.run
```

**Response:**
```
✅ Breakpoint set at com.example.probedemo.ProbeDemoApplication:13
   Method: main
   Breakpoint ID: bp_2
   JDWP Request ID: 2
```

### Step 3: Restart the Application and Hit Breakpoint

After restarting the Java app, the breakpoint will be hit during startup.

**Prompt:**
```
Show me the current stack when the breakpoint hits
```

**Response (example):**
```
🔍 Stack for thread 1 (5 frames):

Frame 0:
  Location: com.example.probedemo.ProbeDemoApplication.main:13
  Variables:
    - args: (String[]) @0x7f8b9c001000

Frame 1:
  Location: jdk.internal.reflect.DirectMethodHandleAccessor.invoke:104
  ...
```

### Step 4: Check ObservationRegistry Bean Status

**Prompt:**
```
List all threads and find the one initializing Spring beans
```

**Response:**
```
🧵 37 thread(s):
  Thread 1 (ID: 0x1) - main
     Status: Has frames (suspended at breakpoint)
  Thread 2 (ID: 0x2) - Reference Handler
  ...
```

### Step 5: Set Breakpoint in Spring Bean Post-Processor

To verify post-processing, we need to catch when beans are being processed.

**Prompt:**
```
Set a breakpoint at org.springframework.context.support.AbstractApplicationContext
line 869 in the refresh method where BeanPostProcessors are registered
```

**Response:**
```
✅ Breakpoint set at org.springframework.context.support.AbstractApplicationContext:869
   Method: refresh
   Breakpoint ID: bp_3
```

### Step 6: Continue and Inspect When Hit

**Prompt:**
```
Continue execution and wait for the next breakpoint
```

When the breakpoint hits:

**Prompt:**
```
Get the current stack with variables and show me the beanFactory field
```

**Expected Response:**
```
🔍 Stack for thread 1 (12 frames):

Frame 0:
  Location: AbstractApplicationContext.refresh:869
  Variables:
    - this: (AbstractApplicationContext) @0x7f8b9c002000
    - beanFactory: (DefaultListableBeanFactory) @0x7f8b9c003000
    - beanPostProcessors: (List) @0x7f8b9c004000
```

### Step 7: Verify ObservationRegistry Registration

**Prompt:**
```
Evaluate this.getBeanNamesForType(ObservationRegistry.class) to see if
ObservationRegistry beans are registered
```

**Response (if working correctly):**
```
Expression evaluated:
["observationRegistry"]
```

**Response (if bug is present):**
```
Expression evaluated:
[] (empty - no ObservationRegistry beans found)
```

### Step 8: Check for Early Initialization Warnings

**Prompt:**
```
Set a breakpoint at org.springframework.beans.factory.support.AbstractAutowireCapableBeanFactory
line 1456 where it logs warnings about beans not eligible for post-processing
```

**Response:**
```
✅ Breakpoint set at AbstractAutowireCapableBeanFactory:1456
   Method: applyBeanPostProcessorsBeforeInitialization
```

### Step 9: Validate Metrics Endpoint

After debugging, verify the fix by checking metrics:

**Prompt:**
```
Resume execution and let the application fully start
```

Then check metrics endpoint:
```bash
curl http://localhost:8080/actuator/metrics/http.server.requests
```

**Expected (working):**
```json
{
  "name": "http.server.requests",
  "measurements": [
    {"statistic": "COUNT", "value": 42.0}
  ]
}
```

**Bug present:**
```json
{
  "error": "Not Found",
  "message": "No such metric: http.server.requests"
}
```

## Key Findings

Using the JDWP debugger, we can verify:

1. **Bean Registration**: Whether ObservationRegistry is registered in the Spring context
2. **Post-Processor Timing**: When BeanPostProcessors run relative to ObservationRegistry initialization
3. **Initialization Order**: If ObservationRegistry is being eagerly initialized before post-processors are ready
4. **Warning Messages**: Catching the exact point where Spring logs post-processing warnings

## Common Issues Found

- **Early Initialization**: ObservationRegistry created before BeanPostProcessors registered
- **Missing Dependencies**: Required beans not available during post-processing
- **Configuration Order**: @Configuration classes loaded in wrong order

## Solution Verification

After applying a fix (e.g., adjusting Spring Security version or bean configuration):

**Prompt:**
```
Clear all breakpoints and disconnect from the debug session
```

**Response:**
```
✅ All breakpoints cleared
✅ Disconnected from debug session
```

Then verify metrics are collecting:
```bash
# Generate some traffic
curl http://localhost:8080/

# Check metrics
curl http://localhost:8080/actuator/metrics/http.server.requests
```

## Validated Approach: Runtime Verification

Since connecting to an already-running application is the most common scenario (especially with production systems via kubectl port-forward), here's how to verify post-processing after the fact:

### Step 1: Verify Metrics Endpoint

```bash
# Generate some traffic
curl http://localhost:8080/

# Check if metrics are being collected
curl http://localhost:8081/actuator/metrics/http.server.requests
```

**Working (post-processing succeeded):**
```json
{
  "name": "http.server.requests",
  "measurements": [{"statistic": "COUNT", "value": 1.0}]
}
```

**Bug present (post-processing failed):**
```json
{
  "error": "Not Found",
  "message": "No such metric: http.server.requests"
}
```

### Step 2: Use Debugger to Inspect Live State

**Prompt:**
```
Attach to localhost:5005
Set a breakpoint at HelloController line 57
```

Trigger a request to hit the breakpoint, then:

**Prompt:**
```
Show me the current stack and the meterRegistry field
List all threads
```

This validates:
- The application responds to breakpoints
- You can inspect the MeterRegistry that was injected
- Metrics are being recorded (proving post-processing worked)

### Step 3: Verify Bean Configuration

If metrics aren't working, check the application logs for warnings:

```
Bean 'observationRegistry' is not eligible for getting processed by all BeanPostProcessors
```

This warning indicates the bug is present.

## Conclusion

This debugging session demonstrates how to:
- Verify ObservationRegistry post-processing via metrics
- Use natural language to set strategic breakpoints
- Inspect application state at runtime
- Validate observability configuration without code changes

The JDWP MCP debugger enables investigating complex framework issues in running applications without writing test code or adding logging statements.

**Key Insight**: For production debugging, verifying the symptoms (metrics work/don't work) is often more practical than trying to replay initialization. The debugger lets you inspect the current state to understand what went wrong.
