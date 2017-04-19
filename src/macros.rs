#[macro_export]
macro_rules! exit_with_error(
    ($($arg:tt)*) => { {
        use std::process;
        writeln!(&mut ::std::io::stderr(), $($arg)*)
            .expect("Failed while printing to stderr");
        process::exit(1);
    } }
);

#[macro_export]
macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);
