#[allow(unused)]
fn parse_output<'a>(delimeter: &str, input: &[&'a str]) -> Vec<&'a str> {
    input
        .iter()
        .flat_map(|s| -> Vec<&str> {
            s.split(delimeter)
                .enumerate()
                .filter_map(|(i, s)| if i % 2 != 0 { Some(s) } else { None })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_output;

    #[test]
    fn splits_basic_code_example() {
        let s = "Example text upper\n```\nExpected Output\n```\nExample text lower";
        let input = vec![s];
        let output = parse_output("```", &input);
        assert_eq!(vec!["\nExpected Output\n"], output);
    }

    #[test]
    fn splits_two_code_examples() {
        let s = "Example text upper\n```\nExpected Output\n```\nExample text lower\n```\nExpected Output 2\n```";
        let input = vec![s];
        let output = parse_output("```", &input);
        assert_eq!(vec!["\nExpected Output\n", "\nExpected Output 2\n"], output);
    }

    #[test]
    fn splits_two_when_text_starts_with_code_example() {
        let s = "```\nExpected Output\n```\nExample text lower\n```\nExpected Output 2\n```";
        let input = vec![s];
        let output = parse_output("```", &input);
        assert_eq!(vec!["\nExpected Output\n", "\nExpected Output 2\n"], output);
    }
}
