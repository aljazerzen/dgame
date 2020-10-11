use gamemath::{Mat3, Vec3};

#[allow(non_snake_case)]
pub fn solve_lu(A: &Mat3, b: Vec3<f32>) -> Vec3<f32> {
  let (L, U) = lu(A);

  let y = solve_L(L, b);

  solve_U(U, y)
}

#[allow(non_snake_case)]
fn solve_L(L: Mat3, b: Vec3<f32>) -> Vec3<f32> {
  let mut y: Vec3<f32> = Vec3::default();

  for i in 0..3 {
    let mut sum = 0.0;
    for j in 0..i {
      sum += y[j] * L[i][j];
    }
    y[i] = b[i] - sum;
  }
  y
}

#[allow(non_snake_case)]
fn solve_U(U: Mat3, y: Vec3<f32>) -> Vec3<f32> {
  let mut x: Vec3<f32> = Vec3::default();

  for i in (0..3).rev() {
    let mut sum = 0.0;
    for j in i + 1..3 {
      sum += x[j] * U[i][j];
    }
    x[i] = (y[i] - sum) / U[i][i];
  }
  x
}

// fn pivot(a: &mut Mat3) {
//   let matrix_dimension = A.rows();
//   let mut P: Array2<T> = Array::eye(matrix_dimension);
//   for (i, column) in A.axis_iter(Axis(1)).enumerate() {
//     // find idx of maximum value in column i
//     let mut max_pos = i;
//     for j in i..matrix_dimension {
//       if column[max_pos].abs() < column[j].abs() {
//         max_pos = j;
//       }
//     }
//     // swap rows of P if necessary
//     if max_pos != i {
//       swap_rows(&mut P, i, max_pos);
//     }
//   }
//   P
// }
// fn swap_rows(A: &mut Mat3, idx_row1: usize, idx_row2: usize) {
//   let row_1 = A[idx_row1];
//   A[idx_row1] = A[idx_row2];
//   A[idx_row2] = row_1;
// }

/// Decomposes matrix A into L and U matrices such that A = L * U where L is lower
/// triangular matrix and U is upper triangular matrix.
/// Also, diagonal of L only contains values of 1.
#[allow(non_snake_case)]
fn lu(A: &Mat3) -> (Mat3, Mat3) {
  let mut L: Mat3 = Mat3::identity();
  let mut U: Mat3 = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0).into();

  for col in 0..3 {
    // fill U
    for row in 0..col + 1 {
      let mut sum = 0.0;
      for i in 0..row {
        sum += U[i][col] * L[row][i];
      }

      U[row][col] = A[row][col] - sum;
    }
    // fill L
    for row in col + 1..3 {
      let mut sum = 0.0;
      for i in 0..col {
        sum += U[i][col] * L[row][i];
      }
      L[row][col] = (A[row][col] - sum) / U[col][col];
    }
  }
  (L, U)
}
