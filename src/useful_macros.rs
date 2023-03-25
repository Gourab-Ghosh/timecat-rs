#[macro_export]
macro_rules! generator {
    ($id1:ident | $id2:ident in $it:expr) => {{
        let mut vec = Vec::new();
        for $id2 in $it {
            vec.push($id1);
        }
        vec
    }};

    ($id1:ident | $id2:ident in $it:expr, $cond:expr) => {{
        let mut vec = Vec::new();
        for $id2 in $it {
            if $cond {
                vec.push($id1);
            }
        }
        vec
    }};
}

#[macro_export]
macro_rules! get_item_unchecked {
    ($vec:expr, $index:expr) => {
        unsafe { *$vec.get_unchecked($index) }
    };
    ($vec:expr, $index1:expr, $index2:expr) => {
        unsafe { *$vec.get_unchecked($index1).get_unchecked($index2) }
    };
}

#[macro_export]
macro_rules! get_item_unchecked_mut {
    ($vec:expr, $index:expr) => {
        unsafe { &mut *$vec.get_unchecked_mut($index) }
    };
}

#[macro_export]
macro_rules! interpolate {
    ($start:expr, $end:expr, $alpha:expr) => {
        ((1.0 - $alpha as f32) * $start as f32 + $alpha as f32 * $end as f32)
    };
}

#[macro_export]
macro_rules! inverse_interpolate {
    ($start:expr, $end:expr, $value:expr) => {
        ($value as f32 - $start as f32) / ($end as f32 - $start as f32)
    };
}

#[macro_export]
macro_rules! match_interpolate {
    ($new_start:expr, $new_end:expr, $old_start:expr, $old_end:expr, $old_value:expr) => {
        interpolate!(
            $new_start as f32,
            $new_end as f32,
            inverse_interpolate!($old_start as f32, $old_end as f32, $old_value as f32)
        )
    };
}