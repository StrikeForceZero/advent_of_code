use glam::UVec2;

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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct MatrixDetails {
    min: UVec2,
    max: UVec2,
}

impl MatrixDetails {
    pub fn from_matrix<T>(matrix: &Vec<Vec<T>>) -> Self {
        let mut min = UVec2::MAX;
        let mut max = UVec2::MIN;
        for (pos, _) in MatrixIterator::new(matrix) {
            min = min.min(pos);
            max = max.max(pos);
        }
        Self {
            min,
            max,
        }
    }
    pub fn min(&self) -> UVec2 {
        self.min
    }
    pub fn max(&self) -> UVec2 {
        self.max
    }
    pub fn width(&self) -> usize {
        self.max.x as usize - self.min.x as usize
    }
    pub fn height(&self) -> usize {
        self.max.y as usize - self.min.y as usize
    }
    pub fn max_index(&self) -> usize {
        self.width() * self.height()
    }
    pub fn is_within_bounds(&self, pos: UVec2) -> bool {
        if pos.x < self.min.x || pos.y < self.min.y {
            return false;
        }
        if pos.x > self.max.x || pos.y > self.max.y {
            return false;
        }
        true
    }
    pub fn pos_from_index(&self, index: usize) -> Option<UVec2> {
        let x = index % self.width();
        let y = index / self.width();
        let pos = UVec2::new(x as u32, y as u32);
        if !self.is_within_bounds(pos) {
            return None;
        }
        Some(pos)
    }
    pub fn index_from_pos(&self, pos: UVec2) -> Option<usize> {
        if !self.is_within_bounds(pos) {
            return None;
        }
        let x = pos.x as usize - self.min.x as usize;
        let y = pos.y as usize - self.min.y as usize;
        Some(y * self.width() + x)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MatrixIterator<'a, T> {
    // TODO: this should be &'a [&'a [T]] but it would be a pita to update the helper functions right now
    matrix: &'a Vec<Vec<T>>,
    index: usize,
}

impl<'a, T> MatrixIterator<'a, T> {
    pub fn new(matrix: &'a Vec<Vec<T>>) -> Self {
        Self {
            matrix,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for MatrixIterator<'a, T> {
    type Item = (UVec2, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        // Calculate row and column based on the index
        let rows = self.matrix.len();
        if rows == 0 {
            return None;
        }

        let Some(cols) = get_matrix_width(self.matrix) else {
            return None;
        };

        if self.index >= rows * cols {
            return None;
        }

        let row = self.index / cols;
        let col = self.index % cols;
        self.index += 1;

        Some((UVec2::new(col as u32, row as u32), &self.matrix[row][col]))
    }
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
