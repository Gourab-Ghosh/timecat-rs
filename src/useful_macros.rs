#[cfg(not(feature = "speed"))]
#[macro_export]
macro_rules! get_item_unchecked {
    ($vec:expr, $index:expr) => {
        &$vec[$index]
    };
    ($vec:expr, $index1:expr, $index2:expr) => {
        &$vec[$index1][$index2]
    };
    ($vec:expr, $index1:expr, $index2:expr, $index3:expr) => {
        &$vec[$index1][$index2][$index3]
    };
}

#[cfg(not(feature = "speed"))]
#[macro_export]
macro_rules! get_item_unchecked_mut {
    ($vec:expr, $index:expr) => {
        &mut $vec[$index]
    };
    ($vec:expr, $index1:expr, $index2:expr) => {
        &mut $vec[$index1][$index2]
    };
    ($vec:expr, $index1:expr, $index2:expr, $index3:expr) => {
        &mut $vec[$index1][$index2][$index3]
    };
}

#[cfg(feature = "speed")]
#[macro_export]
macro_rules! get_item_unchecked {
    ($vec:expr, $index:expr) => {
        unsafe { $vec.get_unchecked($index) }
    };
    ($vec:expr, $index1:expr, $index2:expr) => {
        unsafe { $vec.get_unchecked($index1).get_unchecked($index2) }
    };
    ($vec:expr, $index1:expr, $index2:expr, $index3:expr) => {
        unsafe {
            $vec.get_unchecked($index1)
                .get_unchecked($index2)
                .get_unchecked($index3)
        }
    };
}

#[cfg(feature = "speed")]
#[macro_export]
macro_rules! get_item_unchecked_mut {
    ($vec:expr, $index:expr) => {
        unsafe { $vec.get_unchecked_mut($index) }
    };
    ($vec:expr, $index1:expr, $index2:expr) => {
        unsafe { $vec.get_unchecked_mut($index1).get_unchecked_mut($index2) }
    };
    ($vec:expr, $index1:expr, $index2:expr, $index3:expr) => {
        unsafe {
            $vec.get_unchecked_mut($index1)
                .get_unchecked_mut($index2)
                .get_unchecked_mut($index3)
        }
    };
}

#[macro_export]
macro_rules! interpolate {
    ($start:expr, $end:expr, $alpha:expr) => {
        ((1.0 - ($alpha as f64)) * ($start as f64) + ($alpha as f64) * ($end as f64))
    };
}

#[macro_export]
macro_rules! inverse_interpolate {
    ($start:expr, $end:expr, $value:expr) => {
        (($value as f64) - ($start as f64)) / (($end as f64) - ($start as f64))
    };
}

#[macro_export]
macro_rules! match_interpolate {
    ($new_start:expr, $new_end:expr, $old_start:expr, $old_end:expr, $old_value:expr) => {
        interpolate!(
            $new_start,
            $new_end,
            inverse_interpolate!($old_start, $old_end, $old_value)
        )
    };
}

#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! print_wasm {
    () => {
        gloo::console::log!()
    };
    ($($arg:tt)*) => {
        gloo::console::log!(format!($($arg)*))
    };
}

#[cfg(not(feature = "wasm"))]
#[macro_export]
macro_rules! print_wasm {
    () => {
        print!()
    };
    ($($arg:tt)*) => {
        print!($($arg)*)
    };
}

#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! println_wasm {
    () => {
        gloo::console::log!("\n")
    };
    ($($arg:tt)*) => {
        gloo::console::log!(format!($($arg)*))
    };
}

#[cfg(not(feature = "wasm"))]
#[macro_export]
macro_rules! println_wasm {
    () => {
        println!()
    };
    ($($arg:tt)*) => {
        println!($($arg)*)
    };
}
