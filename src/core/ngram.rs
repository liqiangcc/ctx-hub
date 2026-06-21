pub fn make_search_ngrams(text: &str) -> String {
    let mut grams = Vec::new();
    let mut current = Vec::new();

    for ch in text.chars() {
        if is_cjk(ch) {
            current.push(ch);
        } else {
            push_ngrams(&current, &mut grams);
            current.clear();
        }
    }
    push_ngrams(&current, &mut grams);

    grams.sort();
    grams.dedup();
    grams.into_iter().collect::<Vec<_>>().join(" ")
}

fn push_ngrams(chars: &[char], grams: &mut Vec<String>) {
    for n in 2..=3 {
        if chars.len() < n {
            continue;
        }
        for window in chars.windows(n) {
            grams.push(window.iter().collect());
        }
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0xF900..=0xFAFF
            | 0x3040..=0x30FF
            | 0xAC00..=0xD7AF
    )
}
