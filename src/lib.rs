use std::time::{SystemTime, UNIX_EPOCH};

const MESSAGES: [&str; 6] = [
    "You meant `git`, didn't you? Bold typo.",
    "Cute try. Still looks a lot like `git` was the plan.",
    "That honk was suspiciously close to `git`.",
    "You typed `gut`. Your fingers clearly wanted `git`.",
    "Confidently wrong. Were you aiming for `git`?",
    "Impressive. You missed `git` by one letter.",
];

const GOOSE: &str = r#" _
__(.)<
/___)
 " ""#;

pub fn run() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as usize)
        .unwrap_or(0);

    let message = MESSAGES[now % MESSAGES.len()];
    println!("{GOOSE}\n\n{message}");
}
