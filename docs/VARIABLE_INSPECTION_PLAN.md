# Variable Inspection Implementation Plan

## Goal

Make the JDWP debugger useful for inspecting variable values in both user code and library code (like Spring Boot, Micrometer) to support real-world debugging scenarios.

**Core Use Case**: "Why isn't my custom metric showing up in `/actuator/metrics`?"

Users need to:
1. Set a breakpoint where a metric is registered
2. Navigate to the right thread/frame
3. See the actual values of variables (not just object IDs)
4. Inspect fields of objects (e.g., `meterRegistry` fields)
5. Drill down into collections and nested objects

## Current State

### What Works ✅
- Connect/disconnect to JVM
- Set/clear breakpoints
- List threads
- Get stack frames
- See variable names and types in frames
- Continue/pause execution
- ✅ **String values**: Now show actual string contents (Week 1 complete!)

### What Doesn't Work ❌
- **⚠️ CRITICAL: Breakpoint event visibility**: When a breakpoint hits, users don't know which thread to inspect!
  - Events contain thread ID but MCP server doesn't expose them
  - Users must manually guess which of 36 threads hit the breakpoint
  - Makes the tool unusable for real debugging
- **Object field access**: Can't see fields of an object
- **Collection inspection**: Can't see what's in a List/Map/Set
- **Primitive values in objects**: Primitives in objects aren't accessible
- **Expression evaluation**: Can't call methods like `toString()` or `getMeters()`

## Implementation Phases

### Phase 1: Basic Object Inspection (Highest Priority)

**Goal**: Show useful information about variables without expression evaluation.

#### 1.1 String Value Retrieval
**JDWP Commands**:
- `StringReference.Value` (command set 10, command 1)

**Implementation**:
```rust
// In jdwp-client/src/string.rs
pub async fn get_string_value(&mut self, object_id: ObjectId) -> JdwpResult<String>
```

**User Experience**:
```
Variables:
  - message = (String) "Hello World"  // Instead of @0x123
  - count = (int) 42
```

#### 1.2 Object Field Values
**JDWP Commands**:
- `ReferenceType.Fields` (get field IDs and signatures)
- `ObjectReference.GetValues` (get field values)

**Implementation**:
```rust
// In jdwp-client/src/object.rs
pub async fn get_object_fields(&mut self, object_id: ObjectId) -> JdwpResult<Vec<Field>>

pub struct Field {
    pub name: String,
    pub signature: String,
    pub value: Value,
}
```

**User Experience**:
```
Variables:
  - this = (HelloController) @0x456
    ├─ meterRegistry = (SimpleMeterRegistry) @0x789
    ├─ helloCounter = (Counter) @0xabc
    └─ requestCount = (AtomicInteger) @0xdef
```

#### 1.3 Array/Collection Inspection
**JDWP Commands**:
- `ArrayReference.Length`
- `ArrayReference.GetValues`

**Implementation**:
```rust
// In jdwp-client/src/array.rs
pub async fn get_array_length(&mut self, array_id: ObjectId) -> JdwpResult<i32>
pub async fn get_array_values(&mut self, array_id: ObjectId, first_index: i32, length: i32) -> JdwpResult<Vec<Value>>
```

**User Experience**:
```
Variables:
  - args = (String[]) length=3
    ├─ [0] = "arg1"
    ├─ [1] = "arg2"
    └─ [2] = "arg3"
```

#### 1.4 Enhanced Stack Frame Display
**Update**: Modify `handle_get_stack()` to automatically expand objects.

**Configuration**:
```rust
pub struct InspectionConfig {
    pub max_depth: usize,           // Default: 2
    pub max_collection_items: usize, // Default: 10
    pub auto_expand_strings: bool,   // Default: true
    pub expand_fields: bool,         // Default: true
}
```

**User Experience**:
```
Frame 0: HelloController.hello:57
  Variables:
    - this = (HelloController) @0x456
      ├─ meterRegistry = (SimpleMeterRegistry) @0x789
      │  └─ meters = (ConcurrentHashMap) @0xbcd size=15
      ├─ helloCounter = (Counter) @0xabc
      │  ├─ id = (Meter.Id) @0xef0
      │  │  ├─ name = "hello_requests_total"
      │  │  └─ tags = (List) size=1
      │  └─ count = 42.0
      └─ requestCount = (AtomicInteger) @0xdef value=42
    - sample = (Timer.Sample) @0x111
```

### Phase 2: Smart Object Navigation

**Goal**: Help users find the information they need without knowing JDWP internals.

#### 2.1 Type Information Cache
**Problem**: Every object inspection requires multiple JDWP calls to get type info.

**Solution**: Cache `ReferenceType.Fields` results by class ID.

```rust
// In mcp-server/src/session.rs
pub struct TypeCache {
    fields: HashMap<ReferenceTypeId, Vec<FieldInfo>>,
}
```

#### 2.2 Natural Language Field Access
**Goal**: Users say "show me this.meterRegistry" instead of navigating manually.

**Implementation**: Parse field path and fetch recursively.

```rust
// In mcp-server/src/handlers.rs
async fn handle_inspect_field(&self, args: Value) -> Result<String, String> {
    // Parse "this.meterRegistry.meters"
    // Navigate: get 'this' from frame -> get 'meterRegistry' field -> get 'meters' field
}
```

**User Experience**:
```
> Show me this.meterRegistry.meters
(ConcurrentHashMap) size=15
  ├─ "hello_requests_total" -> (Counter) count=42.0
  ├─ "http.server.requests" -> (Timer) count=5
  └─ ... (10 more items)
```

#### 2.3 Collection Search
**Goal**: Find items in collections without dumping everything.

```rust
async fn handle_find_in_collection(&self, args: Value) -> Result<String, String> {
    // Find meter with name containing "hello"
}
```

**User Experience**:
```
> Find meters with name containing "hello"
Found 2 matches:
  - hello_requests_total (Counter) count=42.0
  - hello_errors_total (Counter) count=3.0
```

### Phase 3: Expression Evaluation (Future)

**Goal**: Allow method calls and complex expressions.

**JDWP Commands**:
- `StackFrame.GetValues` (get variable values in frame)
- `ObjectReference.InvokeMethod` (call methods on objects)
- `ClassType.InvokeMethod` (call static methods)

**Challenges**:
- Expression parsing (could use a Java parser crate)
- Method signature resolution
- Handling exceptions during evaluation
- Thread management (requires suspending threads)

**Deferred**: This is complex and Phase 1-2 provide 80% of the value.

## Implementation Order

### Week 1: Core Infrastructure
1. ✅ Fix INVALID_LENGTH bug
2. ✅ Implement `StringReference.Value` command
3. ✅ Implement `ReferenceType.Fields` command
4. ✅ Implement `ObjectReference.GetValues` command
5. ✅ Auto-expand strings in `handle_get_stack()`
6. **🚨 BLOCKER: Expose breakpoint events to users**
   - Add MCP handler to show last breakpoint event with thread ID
   - Store last event in session
   - Return thread ID when breakpoint hits

### Week 2: Object Inspection
6. Implement recursive object expansion (with max depth)
7. Add type cache for performance
8. Update `handle_get_stack()` to auto-expand objects
9. Test with HelloController (verify we can see meterRegistry fields)

### Week 3: Collections & Polish
10. Implement array inspection commands
11. Add special handling for common types (List, Map, Set, Optional)
12. Add configuration for inspection depth/limits
13. Create actuator metrics debugging example (NOW VALID!)

### Week 4: Advanced Navigation
14. Implement field path navigation ("this.meterRegistry.meters")
15. Add collection search/filter
16. Performance optimization and caching
17. Documentation and examples

## Success Criteria

### Minimum Viable (Phase 1)
A user can:
- Set a breakpoint in HelloController
- See that `meterRegistry` is a SimpleMeterRegistry
- See fields of `meterRegistry` (including `meters` collection)
- See that `helloCounter` exists with count=42.0
- See string values directly (not object IDs)

### Stretch Goal (Phase 2)
A user can:
- Ask "show me this.meterRegistry.meters" and get the map contents
- Ask "find metrics with name containing 'hello'" and get matches
- Navigate complex object graphs without knowing JDWP

## JDWP Commands Reference

### Priority 1 (Phase 1)
| Command | Command Set | ID | Purpose |
|---------|-------------|----|----|
| StringReference.Value | 10 | 1 | Get string contents |
| ReferenceType.Fields | 2 | 4 | Get field info for a class |
| ObjectReference.GetValues | 9 | 2 | Get field values from object |
| ArrayReference.Length | 13 | 1 | Get array length |
| ArrayReference.GetValues | 13 | 2 | Get array elements |

### Priority 2 (Phase 2)
| Command | Command Set | ID | Purpose |
|---------|-------------|----|----|
| ReferenceType.Methods | 2 | 5 | Get method info (for future eval) |
| ObjectReference.ReferenceType | 9 | 1 | Get type of an object |

### Priority 3 (Phase 3 - Future)
| Command | Command Set | ID | Purpose |
|---------|-------------|----|----|
| StackFrame.GetValues | 16 | 1 | Get variable values by slot |
| ObjectReference.InvokeMethod | 9 | 6 | Call methods on objects |
| ClassType.InvokeMethod | 3 | 3 | Call static methods |

## Testing Strategy

### Unit Tests (in jdwp-client)
- Test each JDWP command implementation
- Mock JDWP responses
- Test edge cases (null objects, empty collections)

### Integration Tests (in mcp-server)
- Test against actual Java application (HelloController)
- Verify object expansion works
- Test recursive expansion with depth limits
- Performance testing (large collections, deep nesting)

### Example Validation
- Follow the actuator metrics example step-by-step
- Verify every command in the example works
- Ensure output matches documentation

## Open Questions

1. **How deep should we auto-expand objects?**
   - Proposal: max_depth=2 by default, configurable via args

2. **How to handle circular references?**
   - Proposal: Track visited objects, show "... (circular reference)" when detected

3. **How to handle large collections?**
   - Proposal: Show first 10 items, add "show more" capability

4. **Should we cache object field values?**
   - Proposal: No - values may change, always fetch fresh

5. **How to present nested data in MCP text output?**
   - Proposal: Use tree characters (├─ └─) like current stack output

## Non-Goals

- **Full debugger UI**: We're building a backend, not a GUI
- **Hot code reload**: Out of scope for JDWP
- **Code modification**: Read-only inspection only
- **Performance profiling**: Use a proper profiler
- **Memory leak detection**: Use a proper memory analyzer

## Resources

- [JDWP Specification](https://docs.oracle.com/javase/8/docs/technotes/guides/jpda/jdwp-spec.html)
- [JDI Documentation](https://docs.oracle.com/javase/8/docs/jdk/api/jpda/jdi/) (reference implementation)
- [Existing JDWP client implementations](https://github.com/search?q=jdwp+client)
