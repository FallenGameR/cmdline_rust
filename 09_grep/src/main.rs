fn main() {
    dbg!("Hello, world!"); // can return stuff
    // A Pin<P> ensures that the pointee of any pointer type P has a stable location in memory (self-referencing struct)
    // println!("Initializing an instance of {}", std::any::type_name::<T>());

    // let bar = Some(4);
    // assert!(matches!(bar, Some(x) if x > 2))

    // Subslice patterns
    // fn foo(words: &[&str]) {
    //     match words {
    //         // Ignore everything but the last element, which must be "!".
    //         [.., "!"] => println!("!!!"),
    //
    //         // `start` is a slice of everything except the last element, which must be "z".
    //         [start @ .., "z"] => println!("starts with: {:?}", start),
    //
    //         // `end` is a slice of everything but the first element, which must be "a".
    //         ["a", end @ ..] => println!("ends with: {:?}", end),
    //
    //         rest => println!("{:?}", rest),
    //     }
    // }

    // let exe = env!("PATH");

    // match x as u32 {
    //     0 => println!("zero!"),
    //     1.. => println!("positive number!"),
    // }

    // cargo build --timings

    // let else blocks
}
