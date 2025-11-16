use easy_repl::{Repl, CommandStatus, command};

fn main() {
    println!("Aegisr REPL - Type 'help' for a list of commands.");
    let mut repl = Repl::builder()
        .add("hello", command! {
        "Say hello",
        (name: String) => |name| {
            println!("Hello {}!", name);
            Ok(CommandStatus::Done)
        }
    })
        .add("add", command! {
        "Add X to Y",
        (X:i32, Y:i32) => |x, y| {
            println!("{} + {} = {}", x, y, x + y);
            Ok(CommandStatus::Done)
        }
    })
        .build().expect("Failed to create repl");

    repl.run().expect("Critical REPL error");
}