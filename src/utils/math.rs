use core::ops::{Add, Div, Mul, Sub};

pub fn remap_clamped<T>(value: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T
where
    T: Copy + PartialOrd + Sub<Output = T> + Add<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    if value <= in_min {
        out_min
    } else if value >= in_max {
        out_max
    } else {
        let t = (value - in_min) / (in_max - in_min);
        out_min + t * (out_max - out_min)
    }
}
