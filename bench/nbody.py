from math import sqrt

N = 5
STEPS = 10000000
DT = 0.01

def energy(x, y, z, vx, vy, vz, m):
    e = 0.0
    for i in range(N):
        e += 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i])
        for j in range(i + 1, N):
            dx, dy, dz = x[i] - x[j], y[i] - y[j], z[i] - z[j]
            e -= m[i] * m[j] / sqrt(dx * dx + dy * dy + dz * dz)
    return e

def main():
    x = [0.0, 1.0, 2.0, 3.0, 4.0]
    y = [0.0, 0.5, 1.0, 1.5, 2.0]
    z = [0.0, 0.25, 0.5, 0.75, 1.0]
    vx = [0.0, 0.1, -0.1, 0.2, -0.2]
    vy = [0.0, 0.2, 0.1, -0.1, -0.2]
    vz = [0.0, 0.0, 0.1, 0.0, -0.1]
    m = [10.0, 1.0, 0.5, 0.25, 0.125]
    for _ in range(STEPS):
        for i in range(N):
            for j in range(i + 1, N):
                dx, dy, dz = x[i] - x[j], y[i] - y[j], z[i] - z[j]
                d2 = dx * dx + dy * dy + dz * dz
                mag = DT / (d2 * sqrt(d2))
                vx[i] -= dx * m[j] * mag; vy[i] -= dy * m[j] * mag; vz[i] -= dz * m[j] * mag
                vx[j] += dx * m[i] * mag; vy[j] += dy * m[i] * mag; vz[j] += dz * m[i] * mag
        for i in range(N):
            x[i] += DT * vx[i]; y[i] += DT * vy[i]; z[i] += DT * vz[i]
    print("%.9f" % energy(x, y, z, vx, vy, vz, m))

main()
