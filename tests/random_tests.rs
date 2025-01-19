#[test]
fn test_boolean_and_behavior() {
    #[derive(Default)]
    struct Counter {
        count: usize,
    }

    impl Counter {
        fn increase_count_and_return_true(&mut self) -> bool {
            self.count += 1;
            true
        }
    }

    let mut counter1 = Counter::default();
    let mut counter2 = Counter::default();
    let mut n1 = 0;
    let mut n2 = 0;
    for i in 0..10 {
        if i % 2 == 0 && counter1.increase_count_and_return_true() {
            n1 += 1;
        }
        if counter2.increase_count_and_return_true() && i % 2 == 0 {
            n2 += 1;
        }
    }
    assert_eq!(n1, 5);
    assert_eq!(counter1.count, 5);
    assert_eq!(n2, 5);
    assert_eq!(counter2.count, 10);
}

#[test]
fn test_boolean_or_behavior() {
    #[derive(Default)]
    struct Counter {
        count: usize,
    }

    impl Counter {
        fn increase_count_and_return_false(&mut self) -> bool {
            self.count += 1;
            false
        }
    }

    let mut counter1 = Counter::default();
    let mut counter2 = Counter::default();
    let mut n1 = 0;
    let mut n2 = 0;
    for i in 0..10 {
        if i % 2 == 0 || counter1.increase_count_and_return_false() {
            n1 += 1;
        }
        if counter2.increase_count_and_return_false() || i % 2 == 0 {
            n2 += 1;
        }
    }
    assert_eq!(n1, 5);
    assert_eq!(counter1.count, 5);
    assert_eq!(n2, 5);
    assert_eq!(counter2.count, 10);
}
