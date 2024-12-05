pub trait CharsMatrixToString {
    fn to_string(&self) -> String;
}

impl CharsMatrixToString for Vec<Vec<char>> {
    fn to_string(&self) -> String {
        self.iter()
            .map(|line| line.iter().collect::<String>() + "\n")
            .collect::<String>()
    }
}

impl CharsMatrixToString for Vec<Vec<Option<char>>> {
    fn to_string(&self) -> String {
        self.iter()
            .map(|line| line.iter().map(|c| c.unwrap_or(' ')).collect::<String>() + "\n")
            .collect::<String>()
    }
}

pub trait StringToCharsMatrix {
    fn to_chars_matrix(&self) -> Vec<Vec<char>>;
}

impl StringToCharsMatrix for String {
    fn to_chars_matrix(&self) -> Vec<Vec<char>> {
        self.as_str().to_chars_matrix()
    }
}

impl StringToCharsMatrix for &str {
    fn to_chars_matrix(&self) -> Vec<Vec<char>> {
        self.lines()
            .map(|line| line.chars().collect())
            .collect()
    }
}

pub fn deformat_string(input: &str) -> String {
    input
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .map(|line| line + "\n")
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::utils::string::{deformat_string, StringToCharsMatrix, CharsMatrixToString};

    #[test]
    fn cleanup_sample_string_test() {
        assert_eq!(deformat_string("
            ABC
            123
        "), "ABC\n123\n");
    }

    #[test]
    fn chars_matrix_to_string_test() {
        let matrix = vec![vec!['A', 'B', 'C'], vec!['1', '2', '3']];
        assert_eq!(matrix.to_string(), "ABC\n123\n");
    }

    #[test]
    fn string_to_chars_matrix_test() {
        assert_eq!("ABC\n123\n".to_chars_matrix(), vec![vec!['A', 'B', 'C'], vec!['1', '2', '3']]);
    }
}
