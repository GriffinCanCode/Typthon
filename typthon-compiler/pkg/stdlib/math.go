// Package stdlib - Mathematical functions
package stdlib

import "math"

// Abs returns the absolute value of x
func Abs(x int64) int64 {
	if x < 0 {
		return -x
	}
	return x
}

// AbsFloat returns the absolute value of x
func AbsFloat(x float64) float64 {
	return math.Abs(x)
}

// Pow returns x raised to the power y
func Pow(x, y int64) int64 {
	result := int64(1)
	for i := int64(0); i < y; i++ {
		result *= x
	}
	return result
}

// PowFloat returns x raised to the power y
func PowFloat(x, y float64) float64 {
	return math.Pow(x, y)
}

// Sqrt returns the square root of x
func Sqrt(x float64) float64 {
	return math.Sqrt(x)
}

// Floor returns the largest integer less than or equal to x
func Floor(x float64) int64 {
	return int64(math.Floor(x))
}

// Ceil returns the smallest integer greater than or equal to x
func Ceil(x float64) int64 {
	return int64(math.Ceil(x))
}

// Min returns the smaller of x and y
func Min(x, y int64) int64 {
	if x < y {
		return x
	}
	return y
}

// Max returns the larger of x and y
func Max(x, y int64) int64 {
	if x > y {
		return x
	}
	return y
}
