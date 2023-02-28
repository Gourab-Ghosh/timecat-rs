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
}

#[macro_export]
macro_rules! get_item_unchecked_mut {
    ($vec:expr, $index:expr) => {
        unsafe { &mut *$vec.get_unchecked_mut($index) }
    };
}
