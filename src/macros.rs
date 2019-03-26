#[macro_export]
macro_rules! fh {
    () =>(format!("[{}:{}] ",file!(),line!()));
    ($fmt:expr) =>(format!(concat!("[{}:{}] ",$fmt),file!(),line!()));
    ($fmt:expr, $($arg:tt)*) =>(format!(concat!("[{}:{}] ",$fmt),file!(),line!(),$($arg)*));
}

#[macro_export]
macro_rules! ph {
    () =>(println!("[{}:{}] ",file!(),line!()));
    ($fmt:expr) =>(println!(concat!("[{}:{}] ",$fmt),file!(),line!()));
    ($fmt:expr, $($arg:tt)*) =>(println!(concat!("[{}:{}] ",$fmt),file!(),line!(),$($arg)*));
}
