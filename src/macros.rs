#[macro_export]
macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        uprint!($serial, concat!($fmt, "\n"))
    };
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}

#[macro_export]
macro_rules! send_empty_frames {
    ($can:expr, $len:expr, [$($id:tt),*]) => {
        $(
            crate::utils::send_empty_message($can, $id, $len);
        )*
    };
}

#[macro_export]
macro_rules! handle_can_error {
    ($string:ident, $error_type:expr, $id:expr, $cp_state:expr, $elapsed:expr) => {
        let mut $string: String<U60> = String::new();
        match $error_type {
            CanError::BufferExhausted => {
                uwrite!(
                    $string,
                    "{} - Buffer Exhausted sending frame for {}",
                    $elapsed,
                    $id
                )
                .ok();
                $cp_state.activity_list.push_back($string);
            }
            CanError::ConfigurationFailed => {
                uwrite!(
                    $string,
                    "{} - Configuration Failed sending frame for {}",
                    $elapsed,
                    $id
                )
                .ok();
                $cp_state.activity_list.push_back($string);
            }
            CanError::InvalidFrame => {
                uwrite!(
                    $string,
                    "{} - Invalid Frame sending frame for {}",
                    $elapsed,
                    $id
                )
                .ok();
                $cp_state.activity_list.push_back($string);
            }
            CanError::Timeout => {
                uwrite!($string, "{} - Timeout sending frame for {}", $elapsed, $id).ok();
                $cp_state.activity_list.push_back($string);
            }
        }
    };
}
