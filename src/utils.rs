// SPDX-License-Identifier: Apache-2.0

// https://github.com/wataash/scraps_rs/blob/c5bfd09/src/lib.rs

// -------------------------------------------------------------------------------------------------
/// # 0 macros

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (crate::utils::_log(crate::utils::_LogLevel::Error, file!(), line!(), format_args!($($arg)*));)
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (crate::utils::_log(crate::utils::_LogLevel::Warn, file!(), line!(), format_args!($($arg)*));)
}
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (crate::utils::_log(crate::utils::_LogLevel::Info, file!(), line!(), format_args!($($arg)*));)
}
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (crate::utils::_log(crate::utils::_LogLevel::Debug, file!(), line!(), format_args!($($arg)*));)
}

macro_rules! ret_e {
    // ref: failure-0.1.8/src/macros.rs bail!
    ($($arg:tt)*) => {
        return Err(crate::utils::_err(file!(), line!(), format_args!($($arg)*)))
    }
}

// -------------------------------------------------------------------------------------------------
/// # io

pub(crate) fn y_n(prompt: &str) -> Result<(), failure::Error> {
    info!("{} (y/N)", prompt);
    let mut s = String::new();
    let _len = std::io::stdin()
        .read_line(&mut s)
        .expect("unexpected input"); // XXX
    s = s.trim().to_lowercase();
    if !(s == "y" || s == "yes") {
        return Err(failure::format_err!("canceled"));
    }
    Ok(())
}

// -------------------------------------------------------------------------------------------------
/// # logger

// TODO: https://doc.rust-lang.org/edition-guide/rust-2018/macros/macro-changes.html

#[allow(dead_code)]
#[doc(hidden)]
pub(crate) enum _LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[doc(hidden)]
pub(crate) fn _log(
    level: crate::utils::_LogLevel,
    file: &str,
    line: u32,
    args: std::fmt::Arguments,
) {
    match level {
        _LogLevel::Error => eprintln!("[E] \x1b[31m{}:{} {}\x1b[0m", file, line, args),
        _LogLevel::Warn => eprintln!("[W] \x1b[33m{}:{} {}\x1b[0m", file, line, args),
        _LogLevel::Info => eprintln!("[I] \x1b[34m{}:{} {}\x1b[0m", file, line, args),
        _LogLevel::Debug => eprintln!("[D] \x1b[37m{}:{} {}\x1b[0m", file, line, args),
    }
}

#[doc(hidden)]
pub(crate) fn _err(file: &str, line: u32, args: std::fmt::Arguments) -> failure::Error {
    // let msg = format!("[E] \x1b[31m{}:{} {}\x1b[0m", file, line, args);
    // let err = failure::err_msg(msg);
    // TODO: issue: confusing error message:
    // error[E0277]: `core::fmt::Opaque` cannot be shared between threads safely
    //   --> src/utils.rs:51:15
    //    |
    // 51 |     let err = failure::err_msg(msg);
    //    |               ^^^^^^^^^^^^^^^^ `core::fmt::Opaque` cannot be shared between threads safely
    //    |
    //   ::: /home/wsh/.cargo/registry/src/github.com-1ecc6299db9ec823/failure-0.1.8/src/error_message.rs:11:44
    //    |
    // 11 | pub fn err_msg<D: Display + Debug + Sync + Send + 'static>(msg: D) -> Error {
    //    |                                            ---- required by this bound in `failure::error_message::err_msg`
    //    |
    //    = help: within `[std::fmt::ArgumentV1<'_>]`, the trait `std::marker::Sync` is not implemented for `core::fmt::Opaque`
    //    = note: required because it appears within the type `&core::fmt::Opaque`
    //    = note: required because it appears within the type `std::fmt::ArgumentV1<'_>`
    //    = note: required because it appears within the type `[std::fmt::ArgumentV1<'_>]`
    //    = note: required because of the requirements on the impl of `std::marker::Send` for `&[std::fmt::ArgumentV1<'_>]`
    //    = note: required because it appears within the type `std::fmt::Arguments<'_>`

    let msg = format!("[E] \x1b[31m{}:{} {}\x1b[0m", file, line, args);
    let err = failure::err_msg(msg);
    eprintln!("{}", err);
    err
}

// -------------------------------------------------------------------------------------------------
/// # string

pub(crate) fn partial_str(s: &str, width: usize) -> String {
    if s.len() <= width {
        return s.to_string();
    }
    if width <= 3 {
        return s[..width].to_string();
    }
    format!("{}...", &s[..(width - 3)]).to_string()
}
