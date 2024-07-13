#[cfg(not(feature = "speed"))]
#[macro_export]
macro_rules! get_item_unchecked {
    ($arr:expr, $index:expr $(,)?) => {
        &$arr[$index]
    };

    (const $arr:expr, $index:expr $(,)?) => {
        &const { $arr }[$index]
    };

    ($arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!($arr[$index], $($rest),+)
    };

    (const $arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!(
            get_item_unchecked!(const arr, index),
            $($rest),+,
        )
    };
}

#[cfg(not(feature = "speed"))]
#[macro_export]
macro_rules! get_item_unchecked_mut {
    ($arr:expr, $index:expr $(,)?) => {
        &mut $arr[$index]
    };

    (const $arr:expr, $index:expr $(,)?) => {
        &mut const { $arr }[$index]
    };

    ($arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!($arr[$index], $($rest),+)
    };

    (const $arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!(
            get_item_unchecked_mut!(const arr, index),
            $($rest),+,
        )
    };
}

#[cfg(feature = "speed")]
#[macro_export]
macro_rules! get_item_unchecked {
    ($arr:expr, $index:expr $(,)?) => {
        unsafe { $arr.get_unchecked($index) }
    };

    (const $arr:expr, $index:expr $(,)?) => {
        unsafe { const { $arr }.get_unchecked($index) }
    };

    ($arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!($arr.get_unchecked($index), $($rest),+)
    };

    (const $arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!(
            get_item_unchecked!(const arr, index),
            $($rest),+,
        )
    };
}

#[cfg(feature = "speed")]
#[macro_export]
macro_rules! get_item_unchecked_mut {
    ($arr:expr, $index:expr $(,)?) => {
        unsafe { $arr.get_unchecked_mut($index) }
    };

    (const $arr:expr, $index:expr $(,)?) => {
        unsafe { const { $arr }.get_unchecked_mut($index) }
    };

    ($arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!($arr.get_unchecked_mut($index), $($rest),+)
    };

    (const $arr:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!(
            get_item_unchecked_mut!(const arr, index),
            $($rest),+,
        )
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
