pub fn get_matrix_width<T>(matrix: &Vec<Vec<T>>) -> Option<usize> {
    let mut size = None;
    for line in matrix.iter() {
        if size.is_none() {
            size = Some(line.len());
            continue;
        }
        if size != Some(line.len()) {
            return None;
        }
    }
    size
}

pub fn get_matrix_height<T>(matrix: &Vec<Vec<T>>) -> usize {
    matrix.len()
}

pub fn is_valid_matrix_dimensions<T: Clone + Default>(matrix: &Vec<Vec<T>>) -> bool {
    get_matrix_height(matrix) > 0 && get_matrix_width(matrix).is_some()
}

fn rotate_matrix_90_degrees_checked<T: Clone + Default>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    if is_valid_matrix_dimensions(&matrix) {
        rotate_matrix_90_degrees_unchecked(matrix)
    } else {
        panic!("Invalid matrix dimensions");
    }
}

fn rotate_matrix_90_degrees_unchecked<T: Clone + Default>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    if matrix.is_empty() {
        return matrix.clone();
    }
    let rows = matrix.len();
    let cols = matrix[0].len();

    // Create a new matrix with reversed dimensions
    let mut rotated = vec![vec![T::default(); rows]; cols];

    // Populate the rotated matrix
    for i in 0..rows {
        for j in 0..cols {
            rotated[j][rows - i - 1] = matrix[i][j].clone();
        }
    }

    rotated
}

pub fn rotate_matrix_n_times<T: Clone + Default>(matrix: &Vec<Vec<T>>, n_times: usize) -> Vec<Vec<T>> {
    if n_times > 0 {
        let n_times = n_times - 1;
        let mut matrix = rotate_matrix_90_degrees_checked(matrix);
        for _ in 0..n_times {
            matrix = rotate_matrix_90_degrees_unchecked(&matrix);
        }
        matrix
    } else {
        matrix.clone()
    }
}

pub fn rotate_matrix_90_degrees<T: Clone + Default>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    rotate_matrix_n_times(matrix, 1)
}

pub fn rotate_matrix_180_degrees<T: Clone + Default>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    rotate_matrix_n_times(matrix, 2)
}

pub fn rotate_matrix_270_degrees<T: Clone + Default>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    rotate_matrix_n_times(matrix, 3)
}

/// extends matrix by amount
/// positive amount extends down, negative amount extends up
pub fn extend_matrix_by<T: Clone + Default>(matrix: &mut Vec<Vec<T>>, amount: i32) {
    if amount == 0 {
        return;
    }
    if !is_valid_matrix_dimensions(&matrix) {
        panic!("Invalid matrix dimensions");
    }
    fn fill<T: Clone>(matrix: &mut Vec<T>, amount: usize, value: T) {
        for _ in 0..amount {
            matrix.push(value.clone());
        }
    }

    let (first, second) = match amount.signum() {
        -1 => (1, 3),
        1 => (3, 1),
        // unreachable because of the amount == 0 check at the top
        _ => unreachable!(),
    };
    *matrix = rotate_matrix_n_times(matrix, first);
    for line in matrix.iter_mut() {
        fill(line, amount.abs() as usize, T::default());
    }
    *matrix = rotate_matrix_n_times(matrix, second);
}

/// returns skewed matrix by the amount,
/// positive amount skews down, negative amount skews up
///
/// example:
///     matrix: ABC
///     amount: 1
///     result: A
///              B
///               C
///
pub fn skew_matrix_by<T: Clone + Default>(matrix: &Vec<Vec<T>>, amount: i32) -> Vec<Vec<T>> {
    if amount == 0 {
        return matrix.clone();
    }
    let Some(width) = get_matrix_width(&matrix) else {
        panic!("Invalid matrix dimensions");
    };

    let mut skewed = matrix.iter().map(|line| vec![T::default(); line.len()]).collect::<Vec<_>>();
    let existing_offset = amount.signum() * -1;
    let extend_amount = amount * width as i32 + existing_offset;
    extend_matrix_by(&mut skewed, extend_amount);
    let amount_abs = amount.abs() as usize;
    // Adjust for positive skew
    if amount > 0 {
        for y in 0..matrix.len() {
            for x in 0..matrix[0].len() {
                let offset = x * amount_abs;
                let skewed_y = y + offset;
                skewed[skewed_y][x] = matrix[y][x].clone();
            }
        }
    }
    // Adjust for negative skew
    else {
        for y in 0..matrix.len() {
            for x in 0..matrix[0].len() {
                let offset = x * amount_abs;
                let skewed_y = skewed.len() - 1 - y - offset;
                skewed[skewed_y][x] = matrix[y][x].clone();
            }
        }
    }
    skewed
}

pub fn matrix_diff<T, F>(a: &Vec<Vec<T>>, b: &Vec<Vec<T>>, diff_fn: F) -> Vec<Vec<Option<T>>>
where T: Clone + Default + PartialEq,
    F: Fn(&T, &T) -> T
{
    if a.len() != b.len() {
        panic!("Matrix dimensions do not match");
    }
    let a_width = get_matrix_width(a);
    let b_width = get_matrix_width(b);
    if a_width != b_width {
        panic!("Matrix dimensions do not match");
    }
    let Some(a_width) = a_width else {
        return b
            .iter()
            .map(|line| line
                .iter()
                .map(|item| Some(item.clone()))
                .collect()
            )
            .collect::<Vec<_>>();
    };
    let mut diff = vec![vec![Option::<T>::None; a_width]; a.len()];
    for (y, (a_row, b_row)) in a.iter().zip(b.iter()).enumerate() {
        for (x, (av, bv)) in a_row.iter().zip(b_row.iter()).enumerate() {
            if av == bv {
                continue;
            }
            diff[y][x] = Some(diff_fn(av, bv));
        }
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_matrix_width_test() {
        let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        assert_eq!(get_matrix_width(&matrix), Some(3));
        let matrix: Vec<Vec<i32>> = vec![vec![]];
        assert_eq!(get_matrix_width(&matrix), Some(0));
        let matrix: Vec<Vec<i32>> = vec![];
        assert_eq!(get_matrix_width(&matrix), None);
        let matrix = vec![vec![1], vec![1, 2]];
        assert_eq!(get_matrix_width(&matrix), None);
    }

    #[test]
    fn is_valid_matrix_dimensions_test() {
        let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        assert!(is_valid_matrix_dimensions(&matrix));
        let matrix = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert!(is_valid_matrix_dimensions(&matrix));
        let matrix = vec![vec![1, 2, 3], vec![4, 5]];
        assert!(!is_valid_matrix_dimensions(&matrix));
        let matrix = vec![vec![1, 2, 3]];
        assert!(is_valid_matrix_dimensions(&matrix));
    }

    #[test]
    #[should_panic]
    fn rotate_matrix_90_degrees_checked_test() {
        let matrix = vec![vec![1], vec![1,2]];
        rotate_matrix_90_degrees_checked(&matrix);
    }

    #[test]
    fn rotate_matrix_90_degrees_unchecked_test() {
        let matrix = vec![vec![1,2], vec![1,2]];
        assert_eq!(rotate_matrix_90_degrees_unchecked(&matrix), vec![vec![1,1], vec![2,2]]);
    }

    #[test]
    fn rotate_matrix_90_degrees_single_row_test() {
        let matrix = vec![vec![1,2]];
        assert_eq!(rotate_matrix_90_degrees(&matrix), vec![vec![1], vec![2]]);
    }

    #[test]
    fn rotate_matrix_90_degrees_test() {
        let matrix = vec![vec![1,2], vec![1,2]];
        assert_eq!(rotate_matrix_90_degrees(&matrix), vec![vec![1,1], vec![2,2]]);
    }

    #[test]
    fn rotate_matrix_180_degrees_test() {
        let matrix = vec![vec![1,2], vec![1,2]];
        assert_eq!(rotate_matrix_180_degrees(&matrix), vec![vec![2,1], vec![2,1]]);
    }

    #[test]
    fn rotate_matrix_270_degrees_test() {
        let matrix = vec![vec![1,2], vec![1,2]];
        assert_eq!(rotate_matrix_270_degrees(&matrix), vec![vec![2,2], vec![1,1]]);
    }

    #[test]
    fn extend_matrix_by_test() {
        let mut matrix = vec![vec![1,2,3], vec![4,5,6], vec![7,8,9]];
        extend_matrix_by(&mut matrix, 2);
        assert_eq!(matrix, vec![vec![1,2,3], vec![4,5,6], vec![7,8,9], vec![0,0,0], vec![0,0,0]]);

        let mut matrix = vec![vec![1,2,3], vec![4,5,6], vec![7,8,9]];
        extend_matrix_by(&mut matrix, -2);
        assert_eq!(matrix, vec![vec![0,0,0], vec![0,0,0], vec![1,2,3], vec![4,5,6], vec![7,8,9]]);
    }

    #[test]
    fn skew_matrix_test() {
        let matrix = vec![vec![1,2,3]];
        let matrix = skew_matrix_by(&matrix, 1);
        assert_eq!(matrix, vec![vec![1,0,0], vec![0,2,0], vec![0,0,3]]);


        let matrix = vec![vec![1,2,3]];
        let matrix = skew_matrix_by(&matrix, -1);
        assert_eq!(matrix, vec![vec![0,0,3], vec![0,2,0], vec![1,0,0]]);
    }

    #[test]
    fn matrix_diff_test() {
        let a = vec![vec![1,2,3], vec![4,5,6], vec![7,8,9]];
        let b = vec![vec![1,2,3], vec![4,5,6], vec![7,8,9]];
        assert_eq!(matrix_diff(&a, &b, |a, b| a + b), vec![vec![None; 3], vec![None; 3], vec![None; 3]]);


        let a = vec![vec![1,2,3], vec![4,5,6], vec![7,8,9]];
        let b = vec![vec![2,2,3], vec![4,5,6], vec![7,8,9]];
        assert_eq!(matrix_diff(&a, &b, |a, b| a + b), vec![vec![Some(3), None, None], vec![None; 3], vec![None; 3]]);
    }
}
