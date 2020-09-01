use std::io::Error;

pub fn quit_code<T>(code: i32) -> T {
    std::process::exit(code)
}

pub fn quit_err<T>(err: Error) -> T {
    quit_code(err.raw_os_error().unwrap_or_else(|| 1))
}
