use ctx_hub::core::ngram::make_search_ngrams;

fn cjk(codepoint: u32) -> char {
    char::from_u32(codepoint).expect("valid cjk codepoint")
}

fn contains_gram(text: &str, gram: &str) -> bool {
    make_search_ngrams(text)
        .split_whitespace()
        .any(|item| item == gram)
}

#[test]
fn generates_two_and_three_char_cjk_ngrams() {
    let a = cjk(0x4e0a);
    let b = cjk(0x4e0b);
    let c = cjk(0x6587);

    let text = format!("{a}{b}{c}");
    let gram_ab = format!("{a}{b}");
    let gram_bc = format!("{b}{c}");
    let gram_abc = format!("{a}{b}{c}");

    assert!(contains_gram(&text, &gram_ab));
    assert!(contains_gram(&text, &gram_bc));
    assert!(contains_gram(&text, &gram_abc));
}

#[test]
fn ignores_non_cjk_text() {
    assert!(make_search_ngrams("alpha beta").is_empty());
}

#[test]
fn handles_mixed_text_by_cjk_runs() {
    let a = cjk(0x4e0a);
    let b = cjk(0x4e0b);
    let c = cjk(0x6587);
    let d = cjk(0x6863);

    let text = format!("abc{a}{b}-def{c}{d}");
    let gram_ab = format!("{a}{b}");
    let gram_cd = format!("{c}{d}");

    assert!(contains_gram(&text, &gram_ab));
    assert!(contains_gram(&text, &gram_cd));
}

#[test]
fn deduplicates_repeated_ngrams() {
    let a = cjk(0x4e0a);
    let b = cjk(0x4e0b);

    let text = format!("{a}{b}{a}{b}");
    let gram_ab = format!("{a}{b}");
    let count = make_search_ngrams(&text)
        .split_whitespace()
        .filter(|item| *item == gram_ab)
        .count();

    assert_eq!(count, 1);
}

#[test]
fn short_single_char_cjk_input_has_no_ngrams() {
    let a = cjk(0x4e0a);
    let text = a.to_string();

    assert!(make_search_ngrams(&text).is_empty());
}
