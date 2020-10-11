#[derive(Debug)]
struct Counter {
    value: u64,
}

fn count_system_starting_at_0(mut count: Local<Option<Counter>>) {
    if let None = *count {
        *count = Some(Counter { value: 0 });
    }
    if let Some(the_count) = &mut *count {
        println!("I've been called {:?} times (starting at 0).", the_count);
        the_count.value += 1;
    }
}

fn count_system_starting_at_1000(mut count: Local<Option<Counter>>) {
    if let None = *count {
        *count = Some(Counter { value: 1000 });
    }
    if let Some(the_count) = &mut *count {
        println!("I've been called {:?} time (starting at 1000).", the_count);
        the_count.value += 1;
    }
}


















