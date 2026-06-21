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

#[cfg(test)]
mod tests {
    use super::*;

    fn grams(text: &str) -> Vec<String> {
        make_search_ngrams(text)
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect()
    }

    #[test]
    fn generates_two_and_three_char_chinese_ngrams() {
        let result = grams("支付失败");
        assert!(result.contains(&"支付".to_string()));
        assert!(result.contains(&"付失".to_string()));
        assert!(result.contains(&"失败".to_string()));
        assert!(result.contains(&"支付失".to_string()));
        assert!(result.contains(&"付失败".to_string()));
    }

    #[test]
    fn ignores_non_cjk_text() {
        assert!(make_search_ngrams("payment-service 401").is_empty());
    }

    #[test]
    fn handles_mixed_text_by_cjk_runs() {
        let result = grams("payment支付失败-service订单异常");
        assert!(result.contains(&"支付".to_string()));
        assert!(result.contains(&"失败".to_string()));
        assert!(result.contains(&"订单".to_string()));
        assert!(result.contains(&"异常".to_string()));
    }

    #[test]
    fn deduplicates_repeated_ngrams() {
        let result = grams("支付支付");
        let count = result.iter().filter(|item| item.as_str() == "支付").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn short_single_char_cjk_input_has_no_ngrams() {
        assert!(make_search_ngrams("支").is_empty());
    }
}
