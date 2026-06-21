use ctx_hub::core::ngram::make_search_ngrams;

const CJK_A: &str = "\u{4E0A}";
const CJK_B: &str = "\u{4E0B}";
const CJK_C: &str = "\u{6587}";
const CJK_D: &str = "\u{6863}";

fn contains_gram(text: &str, gram: &str) -> bool {
    make_search_ngrams(text)
        .split_whitespace()
        .any(|item| item == gram)
}

#[test]
fn generates_two_and_three_char_cjk_ngrams() {
    let text = format!("{CJK_A}{CJK_B}{CJK_C}");
    let gram_ab = format!("{CJK_A}{CJK_B}");
    let gram_bc = format!("{CJK_B}{CJK_C}");
    let gram_abc = format!("{CJK_A}{CJK_B}{CJK_C}");

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
    let text = format!("abc{CJK_A}{CJK_B}-def{CJK_C}{CJK_D}");
    let gram_ab = format!("{CJK_A}{CJK_B}");
    let gram_cd = format!("{CJK_C}{CJK_D}");

    assert!(contains_gram(&text, &gram_ab));
    assert!(contains_gram(&text, &gram_cd));
}

#[test]
fn deduplicates_repeated_ngrams() {
    let text = format!("{CJK_A}{CJK_B}{CJK_A}{CJK_B}");
    let gram_ab = format!("{CJK_A}{CJK_B}");
    let count = make_search_ngrams(&text)
        .split_whitespace()
        .filter(|item| *item == gram_ab)
        .count();

    assert_eq!(count, 1);
}

#[test]
fn short_single_char_cjk_input_has_no_ngrams() {
    assert!(make_search_ngrams(CJK_A).is_empty());
}
