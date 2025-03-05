#include <vector>

typedef std::vector<std::vector<int>> matrix;

int matrix_multiply(matrix a, matrix b, matrix c) {
    int i, j, k;
    int sum;
    int n = a.size();
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            sum = 0;
            for (k = 0; k < n; k++) {
                sum += a[i][k] * b[k][j];
            }
            c[i][j] = sum;
        }
    }
    return 0;
}
