#include <math.h>
#include <stdio.h>
#define N 5
#define STEPS 10000000
#define DT 0.01
static double energy(const double* x, const double* y, const double* z, const double* vx, const double* vy, const double* vz, const double* m) {
    double e = 0.0;
    for (int i = 0; i < N; i++) {
        e += 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i]);
        for (int j = i + 1; j < N; j++) {
            double dx = x[i] - x[j], dy = y[i] - y[j], dz = z[i] - z[j];
            e -= m[i] * m[j] / sqrt(dx * dx + dy * dy + dz * dz);
        }
    }
    return e;
}
int main(void) {
    double x[N] = {0.0, 1.0, 2.0, 3.0, 4.0}, y[N] = {0.0, 0.5, 1.0, 1.5, 2.0}, z[N] = {0.0, 0.25, 0.5, 0.75, 1.0};
    double vx[N] = {0.0, 0.1, -0.1, 0.2, -0.2}, vy[N] = {0.0, 0.2, 0.1, -0.1, -0.2}, vz[N] = {0.0, 0.0, 0.1, 0.0, -0.1};
    double m[N] = {10.0, 1.0, 0.5, 0.25, 0.125};
    for (int step = 0; step < STEPS; step++) {
        for (int i = 0; i < N; i++) {
            for (int j = i + 1; j < N; j++) {
                double dx = x[i] - x[j], dy = y[i] - y[j], dz = z[i] - z[j];
                double d2 = dx * dx + dy * dy + dz * dz;
                double mag = DT / (d2 * sqrt(d2));
                vx[i] -= dx * m[j] * mag; vy[i] -= dy * m[j] * mag; vz[i] -= dz * m[j] * mag;
                vx[j] += dx * m[i] * mag; vy[j] += dy * m[i] * mag; vz[j] += dz * m[i] * mag;
            }
        }
        for (int i = 0; i < N; i++) { x[i] += DT * vx[i]; y[i] += DT * vy[i]; z[i] += DT * vz[i]; }
    }
    printf("%.9f\n", energy(x, y, z, vx, vy, vz, m));
    return 0;
}
