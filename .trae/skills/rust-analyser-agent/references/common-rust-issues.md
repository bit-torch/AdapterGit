# Common Rust Issues Catalog

A reference guide for identifying and understanding common Rust problems.

## Error Handling Anti-Patterns

### 1. Unwrap Abuse

**Problem:**
```rust
let file = File::open("config.txt").unwrap();
let config: Config = serde_json::from_reader(file).unwrap();
```

**Why It Is Bad:**
- Panics on any error
- No context about what went wrong
- Bad user experience in production

**Better Approach:**
```rust
let file = File::open("config.txt")
    .context("Failed to open config file")?;
let config: Config = serde_json::from_reader(file)
    .context("Failed to parse config")?;
```

### 2. Silent Error Dropping

**Problem:**
```rust
let _ = fs::remove_file("temp.txt");
```

**Why It Is Bad:**
- Errors are silently ignored
- Makes debugging difficult
- Can mask real problems

**Better Approach:**
```rust
if let Err(e) = fs::remove_file("temp.txt") {
    log::warn!("Failed to remove temp file: {}", e);
}
```

### 3. String Errors

**Problem:**
```rust
fn do_something() -> Result<(), String> {
    Err("something went wrong".to_string())
}
```

**Why It Is Bad:**
- Hard to match on specific errors
- No structured error information
- Difficult to chain/contextualize

**Better Approach:**
```rust
#[derive(Debug, thiserror::Error)]
enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

fn do_something() -> Result<(), MyError> {
    Err(MyError::Io(io::Error::new(...)))
}
```

## Memory and Performance Issues

### 4. Unnecessary Cloning

**Problem:**
```rust
fn process(items: &Vec<String>) {
    for item in items {
        let owned = item.clone();
        // use owned
    }
}
```

**Why It Is Bad:**
- Unnecessary heap allocations
- Performance overhead
- Memory pressure

**Better Approach:**
```rust
fn process(items: &[String]) {
    for item in items {
        // use item directly if possible
        // or clone only when necessary
    }
}
```

### 5. String vs &str Parameters

**Problem:**
```rust
fn greet(name: &String) {
    println!("Hello, {}", name);
}
```

**Why It Is Bad:**
- Forces callers to have a String
- Cannot pass string literals directly
- Less flexible API

**Better Approach:**
```rust
fn greet(name: &str) {
    println!("Hello, {}", name);
}
```

### 6. Vec vs Slice Parameters

**Problem:**
```rust
fn sum(values: &Vec<i32>) -> i32 {
    values.iter().sum()
}
```

**Why It Is Bad:**
- Tied to Vec specifically
- Cannot pass arrays or slices
- Less generic

**Better Approach:**
```rust
fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}
```

## Concurrency Issues

### 7. Blocking in Async

**Problem:**
```rust
async fn fetch_data() -> Result<Data> {
    let data = blocking_file_read().await?; // Blocking!
    Ok(data)
}
```

**Why It Is Bad:**
- Blocks the async runtime
- Reduces throughput
- Can cause deadlocks

**Better Approach:**
```rust
async fn fetch_data() -> Result<Data> {
    let data = tokio::task::spawn_blocking(|| {
        blocking_file_read()
    }).await??;
    Ok(data)
}
```

### 8. Shared Mutable State

**Problem:**
```rust
static mut COUNTER: i32 = 0;

unsafe fn increment() {
    COUNTER += 1;
}
```

**Why It Is Bad:**
- Data races
- Undefined behavior
- Unsafe code risks

**Better Approach:**
```rust
use std::sync::atomic::{AtomicI32, Ordering};

static COUNTER: AtomicI32 = AtomicI32::new(0);

fn increment() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
}
```

### 9. Mutex Guard Across Await

**Problem:**
```rust
async fn process(data: Arc<Mutex<Data>>) {
    let guard = data.lock().unwrap();
    some_async_operation().await; // Holding lock across await!
    // use guard
}
```

**Why It Is Bad:**
- Can deadlock
- Blocks other tasks
- Poor performance

**Better Approach:**
```rust
async fn process(data: Arc<Mutex<Data>>) {
    {
        let guard = data.lock().unwrap();
        // use guard within scope
    } // Lock released here
    some_async_operation().await;
}
```

## API Design Issues

### 10. Missing Trait Implementations

**Problem:**
```rust
pub struct Config {
    pub value: String,
}
// No Debug, Clone, etc.
```

**Why It Is Bad:**
- Hard to debug
- Cannot easily clone
- Poor ergonomics

**Better Approach:**
```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub value: String,
}
```

### 11. Overly Restrictive Lifetimes

**Problem:**
```rust
fn find<'a>(haystack: &'a str, needle: &'a str) -> Option<&'a str> {
    // needle does not need same lifetime as haystack
}
```

**Why It Is Bad:**
- Unnecessary constraints
- Harder to use
- Lifetime errors

**Better Approach:**
```rust
fn find<'a, 'b>(haystack: &'a str, needle: &'b str) -> Option<&'a str> {
    // separate lifetimes
}
```

### 12. Panic in Library Code

**Problem:**
```rust
pub fn parse_config(input: &str) -> Config {
    if input.is_empty() {
        panic!("empty input");
    }
    // ...
}
```

**Why It Is Bad:**
- Library should not panic
- Caller cannot handle error
- Unexpected crashes

**Better Approach:**
```rust
pub fn parse_config(input: &str) -> Result<Config, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    // ...
}
```

## Logic and Correctness Issues

### 13. Integer Overflow

**Problem:**
```rust
let x: u8 = 255;
let y = x + 1; // Overflow in release mode!
```

**Why It Is Bad:**
- Wraps around in release mode
- Panics in debug mode
- Silent bugs

**Better Approach:**
```rust
let x: u8 = 255;
let y = x.checked_add(1).ok_or("overflow")?;
// or use saturating_add, wrapping_add explicitly
```

### 14. Iterator Invalidation Pattern

**Problem:**
```rust
let mut items = vec![1, 2, 3];
for item in &items {
    if *item == 2 {
        items.push(4); // Cannot mutate while iterating!
    }
}
```

**Why It Is Bad:**
- Compilation error
- Logic error if worked around

**Better Approach:**
```rust
let mut items = vec![1, 2, 3];
let to_add: Vec<_> = items
    .iter()
    .filter(|&&x| x == 2)
    .map(|_| 4)
    .collect();
items.extend(to_add);
```

### 15. Match Non-Exhaustive

**Problem:**
```rust
enum Status {
    Active,
    Inactive,
    Pending,
}

fn handle(status: Status) {
    match status {
        Status::Active => println!("active"),
        Status::Inactive => println!("inactive"),
        // Missing Pending!
    }
}
```

**Why It Is Bad:**
- Runtime panic if Pending encountered
- Logic errors

**Better Approach:**
```rust
fn handle(status: Status) {
    match status {
        Status::Active => println!("active"),
        Status::Inactive => println!("inactive"),
        Status::Pending => println!("pending"),
    }
}
```

## Safety Issues

### 16. Unsafe Without Documentation

**Problem:**
```rust
unsafe {
    *ptr = value;
}
```

**Why It Is Bad:**
- No explanation of safety invariants
- Hard to review
- Risk of UB

**Better Approach:**
```rust
// SAFETY: ptr is valid and aligned because ...
unsafe {
    *ptr = value;
}
```

### 17. Transmute Abuse

**Problem:**
```rust
let bytes: [u8; 4] = [0, 0, 0, 0];
let num: u32 = unsafe { std::mem::transmute(bytes) };
```

**Why It Is Bad:**
- Undefined behavior if sizes differ
- Platform-dependent behavior
- Hard to verify correctness

**Better Approach:**
```rust
let bytes: [u8; 4] = [0, 0, 0, 0];
let num = u32::from_le_bytes(bytes);
```

## Testing Issues

### 18. Tests Without Assertions

**Problem:**
```rust
#[test]
fn test_something() {
    let result = do_something();
    println!("{:?}", result);
}
```

**Why It Is Bad:**
- Test always passes
- No verification
- False confidence

**Better Approach:**
```rust
#[test]
fn test_something() {
    let result = do_something();
    assert_eq!(result, expected);
    assert!(result.is_ok());
}
```

### 19. Non-Deterministic Tests

**Problem:**
```rust
#[test]
fn test_random() {
    let value = rand::random::<u32>();
    assert!(value > 0); // Flaky!
}
```

**Why It Is Bad:**
- Test may randomly fail
- Hard to reproduce
- Unreliable CI

**Better Approach:**
```rust
#[test]
fn test_with_seed() {
    let mut rng = StdRng::seed_from_u64(42);
    let value = rng.gen::<u32>();
    assert_eq!(value, expected_value);
}
```

## Documentation Issues

### 20. Public Items Without Docs

**Problem:**
```rust
pub struct Config;

impl Config {
    pub fn new() -> Self { ... }
    pub fn load(path: &str) -> Result<Self> { ... }
}
```

**Why It Is Bad:**
- Users cannot understand API
- No examples
- Poor discoverability

**Better Approach:**
```rust
/// Configuration for the application.
///
/// # Example
///
/// ```
/// let config = Config::new();
/// ```
pub struct Config;

impl Config {
    /// Creates a new default configuration.
    pub fn new() -> Self { ... }
    
    /// Loads configuration from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist or is invalid.
    pub fn load(path: &str) -> Result<Self> { ... }
}
```
