use std::fmt::Display;
use strum::IntoEnumIterator;

pub fn all_enum_variants_display<T: IntoEnumIterator + Display>() -> Vec<String> {
    T::iter().map(|x| x.to_string()).collect()
}

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let mut matrix = vec![vec![0; b.chars().count() + 1]; a.chars().count() + 1];

    for (i, _) in a.chars().enumerate() {
        matrix[i][0] = i;
    }

    for (j, _) in b.chars().enumerate() {
        matrix[0][j] = j;
    }

    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let substitution_cost = if ca == cb { 0 } else { 1 };

            matrix[i + 1][j + 1] = *[
                matrix[i][j + 1] + 1,
                matrix[i + 1][j] + 1,
                matrix[i][j] + substitution_cost,
            ]
            .iter()
            .min()
            .unwrap();
        }
    }

    matrix[a.chars().count()][b.chars().count()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case("abc", "abc", 0)]
    #[case("abc", "abcd", 1)]
    #[case("abc", "ab", 1)]
    #[case("return", "return", 0)]
    #[case("return", "etur", 2)]

    fn test_levenshtein_distance(#[case] a: &str, #[case] b: &str, #[case] expected: usize) {
        assert_eq!(levenshtein_distance(a, b), expected);
    }
}
