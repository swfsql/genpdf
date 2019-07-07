/// format here
#[macro_export]
macro_rules! fh {
    () =>(format!("[{}:{}] ",file!(),line!()));
    ($fmt:expr) =>(format!(concat!("[{}:{}] ",$fmt),file!(),line!()));
    ($fmt:expr, $($arg:tt)*) =>(format!(concat!("[{}:{}] ",$fmt),file!(),line!(),$($arg)*));
}

/// formar err here
#[macro_export]
macro_rules! feh {
    () =>(format_err!("[{}:{}] ",file!(),line!()));
    ($fmt:expr) =>(format_err!(concat!("[{}:{}] ",$fmt),file!(),line!()));
    ($fmt:expr, $($arg:tt)*) =>(format_err!(concat!("[{}:{}] ",$fmt),file!(),line!(),$($arg)*));
}

// with format here
#[macro_export]
macro_rules! wfh {
    () =>(|e| format!("[{}:{}] {}({:?})",file!(),line!(), &e, &e));
    ($fmt:expr) =>(|e| format!(concat!("[{}:{}] {}({:?})",$fmt),file!(),line!(), &e, &e));
    ($fmt:expr, $($arg:tt)*) =>(|e| format!(concat!("[{}:{}] {}({:?})",$fmt),file!(),line!(),&e,&e,$($arg)*));
}

// with format err here
#[macro_export]
macro_rules! wfeh {
    () =>(|e| format_err!("[{}:{}] {}({:?})",file!(),line!(), &e, &e));
    ($fmt:expr) =>(|e| format_err!(concat!("[{}:{}] {}({:?})",$fmt),file!(),line!(), &e, &e));
    ($fmt:expr, $($arg:tt)*) =>(|e| format_err!(concat!("[{}:{}] {}({:?})",$fmt),file!(),line!(),&e,&e,$($arg)*));
}

// print here
// #[macro_export]
// macro_rules! ph {
//     () =>(println!("[{}:{}] ",file!(),line!()));
//     ($fmt:expr) =>(println!(concat!("[{}:{}] ",$fmt),file!(),line!()));
//     ($fmt:expr, $($arg:tt)*) =>(println!(concat!("[{}:{}] ",$fmt),file!(),line!(),$($arg)*));
// }
