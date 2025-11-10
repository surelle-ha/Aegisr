use crate::modules::termenu::Termenu;

///
/// =========================================
/// Note: This is an internal command intended
/// for development use only. You may remove it
/// depending on your specific use case or how
/// you plan to use this tool.
/// =========================================
///
pub fn register() -> Termenu {
    let command: Termenu = Termenu::new_command(
        "helloworld",
        "Print Hello, World!",
        |_options: &std::collections::HashMap<String, Option<String>>| {
            println!("Hello, World!");
            Ok(())
        },
    );

    command
}
