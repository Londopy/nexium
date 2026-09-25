const N: usize = 5;
const STEPS: i32 = 10000000;
const DT: f64 = 0.01;

fn energy(x: &[f64], y: &[f64], z: &[f64], vx: &[f64], vy: &[f64], vz: &[f64], m: &[f64]) -> f64 {
    let mut e = 0.0;
    for i in 0..N {
        e += 0.5 * m[i] * (vx[i] * vx[i] + vy[i] * vy[i] + vz[i] * vz[i]);
        for j in i + 1..N {
            let (dx, dy, dz) = (x[i] - x[j], y[i] - y[j], z[i] - z[j]);
            e -= m[i] * m[j] / (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    e
}

fn main() {
    let mut x: [f64; N] = [0.0, 1.0, 2.0, 3.0, 4.0];
    let mut y: [f64; N] = [0.0, 0.5, 1.0, 1.5, 2.0];
    let mut z: [f64; N] = [0.0, 0.25, 0.5, 0.75, 1.0];
    let mut vx: [f64; N] = [0.0, 0.1, -0.1, 0.2, -0.2];
    let mut vy: [f64; N] = [0.0, 0.2, 0.1, -0.1, -0.2];
    let mut vz: [f64; N] = [0.0, 0.0, 0.1, 0.0, -0.1];
    let m: [f64; N] = [10.0, 1.0, 0.5, 0.25, 0.125];
    for _ in 0..STEPS {
        for i in 0..N {
            for j in i + 1..N {
                let (dx, dy, dz) = (x[i] - x[j], y[i] - y[j], z[i] - z[j]);
                let d2 = dx * dx + dy * dy + dz * dz;
                let mag = DT / (d2 * d2.sqrt());
                vx[i] -= dx * m[j] * mag; vy[i] -= dy * m[j] * mag; vz[i] -= dz * m[j] * mag;
                vx[j] += dx * m[i] * mag; vy[j] += dy * m[i] * mag; vz[j] += dz * m[i] * mag;
            }
        }
        for i in 0..N { x[i] += DT * vx[i]; y[i] += DT * vy[i]; z[i] += DT * vz[i]; }
    }
    println!("{:.9}", energy(&x, &y, &z, &vx, &vy, &vz, &m));
}
