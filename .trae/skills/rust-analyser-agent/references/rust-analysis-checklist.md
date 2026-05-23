# Rust Analysis Checklist

Use this checklist during manual code review to ensure comprehensive analysis.

## 1. Error Handling

### unwrap() and expect() Usage
- [ ] Search for `.unwrap()` calls - should they be handled gracefully?
- [ ] Search for `.expect()` calls - are the messages helpful?
- [ ] Check if Results are silently ignored with `let _ =`
- [ ] Verify panic messages in `expect()` are descriptive

### Result and Option Handling
- [ ] Are Results propagated correctly with `?` operator?
- [ ] Are `Option` unwraps justified?
- [ ] Check for `Result` types that are never used
- [ ] Look for `unwrap_or`/`unwrap_or_else` opportunities

### Custom Error Types
- [ ] Do custom errors implement `std::error::Error`?
- [ ] Are error messages user-friendly?
- [ ] Is `thiserror` or `anyhow` used appropriately?

## 2. Memory Safety

### Unsafe Code
- [ ] Is `unsafe` usage minimized?
- [ ] Are unsafe blocks properly documented with `// SAFETY:` comments?
- [ ] Are invariants maintained across unsafe boundaries?
- [ ] Check for raw pointer dereferences

### Lifetimes
- [ ] Are lifetime annotations necessary and correct?
- [ ] Check for lifetime elision opportunities
- [ ] Look for potential use-after-free patterns

### Ownership and Borrowing
- [ ] Are clones necessary? Could references be used?
- [ ] Check for unnecessary `to_string()` calls
- [ ] Look for `&Vec<T>` parameters that could be `&[T]`
- [ ] Verify `&String` parameters that could be `&str`

## 3. Concurrency

### Thread Safety
- [ ] Are shared state accesses properly synchronized?
- [ ] Check for `Send` and `Sync` implementations
- [ ] Look for potential deadlocks
- [ ] Verify `Mutex` usage - is poisoning handled?

### Async/Await
- [ ] Are futures properly awaited?
- [ ] Check for `await` inside loops that could use `join!` or `select!`
- [ ] Look for blocking operations in async contexts
- [ ] Verify cancellation safety

### Channels and Communication
- [ ] Are channels properly closed?
- [ ] Check for send/recv without timeout that could hang
- [ ] Look for channel capacity issues

## 4. Performance

### Allocations
- [ ] Check for unnecessary `Vec` allocations
- [ ] Look for `String` concatenation in loops
- [ ] Verify `with_capacity` is used when size is known
- [ ] Check for boxed trait objects that could be generic

### Iterators
- [ ] Are iterator chains efficient?
- [ ] Look for `collect()` followed by immediate iteration
- [ ] Check for `iter()` vs `into_iter()` vs `iter_mut()` usage
- [ ] Verify `filter_map` vs `filter` + `map` usage

### Collections
- [ ] Is the right collection type used? (Vec vs HashMap vs BTreeMap)
- [ ] Check for O(n) operations in loops
- [ ] Look for unnecessary sorting
- [ ] Verify hash map hasher choice

## 5. API Design

### Visibility
- [ ] Are public items intentionally public?
- [ ] Check for `pub` on implementation details
- [ ] Look for missing `pub(crate)` where appropriate

### Traits
- [ ] Are trait bounds necessary and minimal?
- [ ] Check for trait object safety
- [ ] Look for missing trait implementations (Debug, Clone, etc.)
- [ ] Verify trait coherence rules

### Documentation
- [ ] Are public APIs documented?
- [ ] Do examples in docs compile?
- [ ] Check for `// TODO` and `// FIXME` comments
- [ ] Look for undocumented panics

## 6. Correctness

### Logic Errors
- [ ] Check boundary conditions (off-by-one errors)
- [ ] Verify integer overflow handling
- [ ] Look for incorrect comparisons (e.g., `==` vs `=`)
- [ ] Check for unreachable code

### Type Safety
- [ ] Are newtypes used for domain types?
- [ ] Check for `as` casts that could fail
- [ ] Look for `From`/`Into` implementations
- [ ] Verify `match` exhaustiveness

### Resource Management
- [ ] Are resources properly released?
- [ ] Check for `Drop` implementations where needed
- [ ] Look for RAII patterns
- [ ] Verify file handles are closed

## 7. Idiomatic Rust

### Language Features
- [ ] Are `if let` and `while let` used appropriately?
- [ ] Check for `match` vs `if let` usage
- [ ] Look for `?` operator opportunities
- [ ] Verify `matches!` macro usage

### Standard Library
- [ ] Are standard library types used idiomatically?
- [ ] Check for `Cow` usage for zero-copy
- [ ] Look for `OnceCell`/`LazyLock` for lazy initialization
- [ ] Verify `Default` implementation usage

### Patterns
- [ ] Is builder pattern used for complex construction?
- [ ] Check for `FromStr` vs `parse()` usage
- [ ] Look for `Display` vs `ToString`
- [ ] Verify `AsRef`/`AsMut` usage

## 8. Testing

### Test Coverage
- [ ] Are edge cases tested?
- [ ] Check for error path testing
- [ ] Look for property-based tests where appropriate
- [ ] Verify async tests use proper runtime

### Test Quality
- [ ] Are tests independent and isolated?
- [ ] Check for proper assertions (not just `assert!`)
- [ ] Look for test helper functions
- [ ] Verify `#[should_panic]` tests have expected messages

## 9. Security

### Input Validation
- [ ] Are user inputs validated?
- [ ] Check for buffer overflow risks
- [ ] Look for injection vulnerabilities
- [ ] Verify path traversal protections

### Cryptography
- [ ] Are crypto libraries used correctly?
- [ ] Check for constant-time comparisons
- [ ] Look for secure random generation
- [ ] Verify key handling

### Secrets
- [ ] Are secrets zeroed from memory?
- [ ] Check for secret logging
- [ ] Look for hardcoded credentials
- [ ] Verify debug implementations do not leak secrets

## 10. Maintainability

### Code Organization
- [ ] Are modules well-organized?
- [ ] Check for file length (consider splitting >500 lines)
- [ ] Look for circular dependencies
- [ ] Verify feature flags are used appropriately

### Naming
- [ ] Do names follow Rust conventions?
- [ ] Check for descriptive variable names
- [ ] Look for consistent naming patterns
- [ ] Verify type names are descriptive

### Comments
- [ ] Are complex algorithms explained?
- [ ] Check for outdated comments
- [ ] Look for commented-out code
- [ ] Verify `unsafe` blocks have safety comments
