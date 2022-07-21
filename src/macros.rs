/// Clears the terminal output
/// and sets the cursor to the top left corner
macro_rules! clear {
    () => {
        print!("\x1B[2J\x1B[1;1H");
    };
}

pub(crate) use clear; // Fixes unusable macro: <https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files>