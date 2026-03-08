#[allow(unused)]
pub(super) fn normalize_reading(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c >= 'ア' && c <= 'ン' {
                // Katakana range
                if let Some(x) = char::from_u32(c as u32 - 0x60) {
                    x // Convert to Hiragana
                } else {
                    c // Ensure it's a valid char
                }
            } else {
                c // Return the character as is if it's not in the range
            }
        })
        .collect()
}
