package main

import (
	"fmt"
	"math"
)

const n = 5
const steps = 200000
const dt = 0.01

func energy(x, y, z, vx, vy, vz, m *[n]float64) float64 {
	e := 0.0
	for i := 0; i < n; i++ {
		e += 0.5 * m[i] * (vx[i]*vx[i] + vy[i]*vy[i] + vz[i]*vz[i])
		for j := i + 1; j < n; j++ {
			dx, dy, dz := x[i]-x[j], y[i]-y[j], z[i]-z[j]
			e -= m[i] * m[j] / math.Sqrt(dx*dx+dy*dy+dz*dz)
		}
	}
	return e
}

func main() {
	x := [n]float64{0.0, 1.0, 2.0, 3.0, 4.0}
	y := [n]float64{0.0, 0.5, 1.0, 1.5, 2.0}
	z := [n]float64{0.0, 0.25, 0.5, 0.75, 1.0}
	vx := [n]float64{0.0, 0.1, -0.1, 0.2, -0.2}
	vy := [n]float64{0.0, 0.2, 0.1, -0.1, -0.2}
	vz := [n]float64{0.0, 0.0, 0.1, 0.0, -0.1}
	m := [n]float64{10.0, 1.0, 0.5, 0.25, 0.125}
	for step := 0; step < steps; step++ {
		for i := 0; i < n; i++ {
			for j := i + 1; j < n; j++ {
				dx, dy, dz := x[i]-x[j], y[i]-y[j], z[i]-z[j]
				d2 := dx*dx + dy*dy + dz*dz
				mag := dt / (d2 * math.Sqrt(d2))
				vx[i] -= dx * m[j] * mag
				vy[i] -= dy * m[j] * mag
				vz[i] -= dz * m[j] * mag
				vx[j] += dx * m[i] * mag
				vy[j] += dy * m[i] * mag
				vz[j] += dz * m[i] * mag
			}
		}
		for i := 0; i < n; i++ {
			x[i] += dt * vx[i]
			y[i] += dt * vy[i]
			z[i] += dt * vz[i]
		}
	}
	fmt.Printf("%.9f\n", energy(&x, &y, &z, &vx, &vy, &vz, &m))
}
