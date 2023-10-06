# Notes

```rust
// dbg! can trace in a useful ways what happens in the program
// it can return stuff and be invoked without arguments
dbg!("Hello, world!"); 

// A Pin<P> ensures that the pointee of any pointer type P has a stable location in memory (useful for self-referencing structs)

// trace type expanded from a generic
println!("Initializing an instance of {}", std::any::type_name::<T>());

// matches! can be used for pattern matching that can be resolved to either true or false
// useful in if conditions when doing a full match stament block seems like too much
let bar = Some(4);
assert!(matches!(bar, Some(x) if x > 2))

// Subslice patterns
// slices are very veratile to hold the data
fn foo(words: &[&str]) {
    match words {
        // Ignore everything but the last element, which must be "!".
        [.., "!"] => println!("!!!"),
        // `start` is a slice of everything except the last element, which must be "z".
        [start @ .., "z"] => println!("starts with: {:?}", start),
        // `end` is a slice of everything but the first element, which must be "a".
        ["a", end @ ..] => println!("ends with: {:?}", end),
        rest => println!("{:?}", rest),
    }
}

// matching of integers with ranges
match x as u32 {
    0 => println!("zero!"),
    1.. => println!("positive number!"),
}

// this should be useful for getting cargo-populate environment variables
// unclear if it is better compared to library style environment variable retrieval
let exe = env!("PATH");

// let else blocks for fast failing
// sugaring of the if let block
let Ok(count) = u64::from_str(count_str) else {
  panic!("Can't parse integer: '{count_str}'");
};
```

```ps1
# Check build timings
cargo build --timings

# Auto fix non-idiomatic errors
cargo clippy --fix -- --warn clippy::pedantic --allow clippy::missing-errors-doc --allow clippy::missing-panics-doc --allow clippy::needless-pass-by-value 

# Generate docs site for your code
cargo doc --open --document-private-items

# Update all dependencies to the latest versions
# Guidance after 1.72 is to commit the cargo.lock file for build repeatability
cargo update
```
