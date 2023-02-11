#![macro_use]

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

// macro_rules! input {
//     ($q:expr) => {
//         {
//             println!("{}", $q);
//             let mut user_input = String::new();
//             std::io::stdin()
//                 .read_line(&mut user_input)
//                 .expect("Failed to read line!");
//             user_input;
//         }
//     };
// }
