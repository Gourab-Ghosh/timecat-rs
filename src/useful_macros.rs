#[cfg(debug_assertions)]
#[macro_export]
macro_rules! get_item_unchecked {
    (@internal $indexable:expr, $index:expr $(,)?) => {
        $indexable[$index]
    };

    (@internal const $indexable:expr, $index:expr $(,)?) => {
        const { $indexable }[$index]
    };

    (@internal $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!(
            @internal
            get_item_unchecked!(@internal $indexable, $index),
            $($rest),+,
        )
    };

    (@internal const $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!(
            @internal
            get_item_unchecked!(@internal const $indexable, $index),
            $($rest),+,
        )
    };

    ($($arg:tt)*) => {
        &get_item_unchecked!(@internal $($arg)*)
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! get_item_unchecked_mut {
    (@internal $indexable:expr, $index:expr $(,)?) => {
        $indexable[$index]
    };

    (@internal const $indexable:expr, $index:expr $(,)?) => {
        const { $indexable }[$index]
    };

    (@internal $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!(
            @internal
            get_item_unchecked_mut!(@internal $indexable, $index),
            $($rest),+,
        )
    };

    (@internal const $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!(
            @internal
            get_item_unchecked_mut!(@internal const $indexable, $index),
            $($rest),+,
        )
    };

    ($($arg:tt)*) => {
        &mut get_item_unchecked_mut!(@internal $($arg)*)
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! get_item_unchecked {
    (@internal $indexable:expr, $index:expr $(,)?) => {
        $indexable.get_unchecked($index)
    };

    (@internal const $indexable:expr, $index:expr $(,)?) => {
        const { $indexable }.get_unchecked($index)
    };

    (@internal $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!(
            @internal
            get_item_unchecked!(@internal $indexable, $index),
            $($rest),+,
        )
    };

    (@internal const $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked!(
            @internal
            get_item_unchecked!(@internal const $indexable, $index),
            $($rest),+,
        )
    };

    ($($arg:tt)*) => {
        unsafe { get_item_unchecked!(@internal $($arg)*) }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! get_item_unchecked_mut {
    (@internal $indexable:expr, $index:expr $(,)?) => {
        $indexable.get_unchecked_mut($index)
    };

    (@internal const $indexable:expr, $index:expr $(,)?) => {
        const { $indexable }.get_unchecked_mut($index)
    };

    (@internal $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!(
            @internal
            get_item_unchecked_mut!(@internal $indexable, $index),
            $($rest),+,
        )
    };

    (@internal const $indexable:expr, $index:expr, $($rest:expr),+ $(,)?) => {
        get_item_unchecked_mut!(
            @internal
            get_item_unchecked_mut!(@internal const $indexable, $index),
            $($rest),+,
        )
    };

    ($($arg:tt)*) => {
        unsafe { get_item_unchecked_mut!(@internal $($arg)*) }
    };
}

#[macro_export]
macro_rules! interpolate {
    (@internal $start:expr, $end:expr, $alpha:expr) => {
        (1.0 - $alpha) * $start + $alpha * $end
    };

    ($start:expr, $end:expr, $alpha:expr $(,)?) => {
        {
            let start = $start as f64;
            let end = $end as f64;
            let alpha = $alpha as f64;
            interpolate!(@internal start, end, alpha)
        }
    };
}

#[macro_export]
macro_rules! inverse_interpolate {
    (@internal $start:expr, $end:expr, $value:expr) => {
        ($value - $start) / ($end - $start)
    };

    ($start:expr, $end:expr, $value:expr $(,)?) => {
        {
            let start = $start as f64;
            let end = $end as f64;
            let value = $value as f64;
            inverse_interpolate!(@internal start, end, alpha)
        }
    };
}

#[macro_export]
macro_rules! match_interpolate {
    ($new_start:expr, $new_end:expr, $old_start:expr, $old_end:expr, $old_value:expr $(,)?) => {
        {
            let new_start = $new_start as f64;
            let new_end = $new_end as f64;
            let old_start = $old_start as f64;
            let old_end = $old_end as f64;
            let old_value = $old_value as f64;
            interpolate!(
                @internal
                new_start,
                new_end,
                (inverse_interpolate!(@internal old_start, old_end, old_value))
            )
        }
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
