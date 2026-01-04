use std::time::Duration;

use crate::{ColorOkLab, EdgeInsets};

#[derive(Debug, Clone)]
pub struct Tween<T, V> {
    t: T,
    start_value: V,
    current_value: V,
    target_value: V,
    status: AnimationStatus,
    duration: T,
    curve_fn: fn(t: T) -> T,
}

#[derive(Debug, Clone)]
pub struct Damp<T, V> {
    speed: T,
    current_value: V,
    target_value: V,
    status: AnimationStatus,
    threshold: T,
    curve_fn: fn(t: T) -> T,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AnimationStatus {
    Idle,
    Started,
    Updated,
    Ended,
}

pub trait Animation<T> {
    fn step(&mut self, delta_time: T);

    fn in_progress(&self) -> bool;
}

pub trait Lerp<T> {
    fn lerp(self, to: Self, t: T) -> Self;
}

impl Lerp<f32> for f32 {
    fn lerp(self, to: f32, t: f32) -> Self {
        (self * (1.0 - t)) + (to * t)
    }
}

impl Lerp<f32> for f64 {
    fn lerp(self, to: f64, t: f32) -> Self {
        (self * (1.0 - t) as f64) + (to * t as f64)
    }
}

impl Lerp<f32> for EdgeInsets {
    fn lerp(self, to: Self, t: f32) -> Self {
        EdgeInsets {
            top: f32::lerp(self.top, to.top, t),
            left: f32::lerp(self.left, to.left, t),
            right: f32::lerp(self.right, to.right, t),
            bottom: f32::lerp(self.bottom, to.bottom, t),
        }
    }
}

impl Lerp<f32> for ColorOkLab {
    fn lerp(self, to: Self, t: f32) -> Self {
        ColorOkLab {
            l: f64::lerp(self.l, to.l, t),
            a: f64::lerp(self.a, to.a, t),
            b: f64::lerp(self.b, to.b, t),
        }
    }
}

impl<V> Default for Tween<f32, V>
where
    V: Default,
{
    fn default() -> Self {
        Self {
            t: 1.,
            start_value: V::default(),
            current_value: V::default(),
            target_value: V::default(),
            status: AnimationStatus::Idle,
            curve_fn: curves::f32::ease_out_quad,
            duration: Duration::from_millis(300).as_secs_f32(),
        }
    }
}

impl<V> Tween<f32, V>
where
    V: Lerp<f32> + Clone,
{
    pub fn new(value: V) -> Self {
        Self {
            t: 1.,
            start_value: value.clone(),
            current_value: value.clone(),
            target_value: value,
            status: AnimationStatus::Idle,
            curve_fn: curves::f32::ease_out_quad,
            duration: Duration::from_millis(300).as_secs_f32(),
        }
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration.as_secs_f32();

        self
    }

    pub fn curve(mut self, curve_fn: fn(t: f32) -> f32) -> Self {
        self.curve_fn = curve_fn;

        self
    }

    pub fn reset(&mut self) {
        self.t = 0.;
        self.status = AnimationStatus::Started;
    }

    pub fn status(&self) -> AnimationStatus {
        self.status
    }

    pub fn tween_to(&mut self, target: V) {
        self.t = 0.;
        self.start_value = self.current_value.clone();
        self.target_value = target;
        self.status = AnimationStatus::Started;
    }

    pub fn set(&mut self, target: V) {
        self.t = 1.;
        self.target_value = target;
        self.start_value = self.target_value.clone();
        self.current_value = self.target_value.clone();

        if self.status == AnimationStatus::Updated {
            self.status = AnimationStatus::Ended;
        } else {
            self.status = AnimationStatus::Idle;
        }
    }

    pub fn t(&self) -> f32 {
        self.t
    }

    pub fn value(&self) -> V {
        self.current_value.clone()
    }
}

impl<V> Animation<f32> for Tween<f32, V>
where
    V: Lerp<f32> + Clone,
{
    fn step(&mut self, delta_time: f32) {
        if self.status == AnimationStatus::Ended {
            self.status = AnimationStatus::Idle;
            return;
        }

        if self.status == AnimationStatus::Idle {
            return;
        }

        self.t += delta_time / self.duration;

        if self.t >= 1. {
            self.t = 1.;
            self.current_value = self.target_value.clone();
            self.status = AnimationStatus::Ended;
        } else {
            self.status = AnimationStatus::Updated;
            self.current_value = V::lerp(
                self.start_value.clone(),
                self.target_value.clone(),
                (self.curve_fn)(self.t),
            );
        }
    }

    fn in_progress(&self) -> bool {
        self.status != AnimationStatus::Idle
    }
}

impl<V> Damp<f32, V>
where
    V: Lerp<f32> + Clone,
{
    pub fn new(value: V) -> Self {
        Self {
            speed: 10.,
            current_value: value.clone(),
            target_value: value,
            curve_fn: decay_curves::f32::default,
            status: AnimationStatus::Idle,
            threshold: 0.01,
        }
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn curve(mut self, curve_fn: fn(t: f32) -> f32) -> Self {
        self.curve_fn = curve_fn;
        self
    }

    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn set(&mut self, value: V) {
        self.current_value = value.clone();
        self.target_value = value;

        if self.status == AnimationStatus::Updated {
            self.status = AnimationStatus::Ended;
        } else {
            self.status = AnimationStatus::Idle;
        }
    }

    pub fn status(&self) -> AnimationStatus {
        self.status
    }

    pub fn value(&self) -> V {
        self.current_value.clone()
    }
}

impl<V> Damp<f32, V>
where
    V: Lerp<f32> + Clone + Distance,
{
    pub fn approach(&mut self, target: V) {
        if self.current_value.distance(&target) > self.threshold {
            self.target_value = target;
            self.status = AnimationStatus::Started;
        } else {
            // Already close, snap to target and stop
            self.target_value = target.clone();
            self.current_value = target;

            if self.status == AnimationStatus::Updated || self.status == AnimationStatus::Started {
                self.status = AnimationStatus::Ended;
            } else {
                self.status = AnimationStatus::Idle;
            }
        }
    }
}

impl<V> Animation<f32> for Damp<f32, V>
where
    V: Lerp<f32> + Clone + Distance,
{
    fn step(&mut self, delta_time: f32) {
        if self.status == AnimationStatus::Ended {
            self.status = AnimationStatus::Idle;
            return;
        }

        if self.status == AnimationStatus::Idle {
            return;
        }

        let distance = self.current_value.distance(&self.target_value);

        if distance < self.threshold {
            self.current_value = self.target_value.clone();
            self.status = AnimationStatus::Ended;
        } else {
            let t = (self.curve_fn)(self.speed * delta_time);
            self.current_value = V::lerp(self.current_value.clone(), self.target_value.clone(), t);
            self.status = AnimationStatus::Updated;
        }
    }

    fn in_progress(&self) -> bool {
        self.status != AnimationStatus::Idle
    }
}

pub trait Distance {
    fn distance(&self, other: &Self) -> f32;
}

impl Distance for f32 {
    fn distance(&self, other: &Self) -> f32 {
        (self - other).abs()
    }
}

impl Distance for EdgeInsets {
    fn distance(&self, other: &Self) -> f32 {
        (self.top - other.top).abs()
            + (self.left - other.left).abs()
            + (self.right - other.right).abs()
            + (self.bottom - other.bottom).abs()
    }
}

pub mod curves {
    pub mod f32 {
        // Linear
        pub fn linear(t: f32) -> f32 {
            t
        }

        pub fn smooth_step(t: f32) -> f32 {
            t * t * (3. - 2. * t)
        }

        pub fn smoother_step(t: f32) -> f32 {
            t * t * t * (t * (6. * t - 15.) + 10.)
        }

        // Quadratic
        pub fn ease_in_quad(t: f32) -> f32 {
            t * t
        }

        pub fn ease_out_quad(t: f32) -> f32 {
            1. - (1. - t) * (1. - t)
        }

        pub fn ease_in_out_quad(t: f32) -> f32 {
            if t < 0.5 {
                2. * t * t
            } else {
                1. - (-2. * t + 2.).powi(2) / 2.
            }
        }

        // Cubic
        pub fn ease_in_cubic(t: f32) -> f32 {
            t * t * t
        }

        pub fn ease_out_cubic(t: f32) -> f32 {
            1. - (1. - t).powi(3)
        }

        pub fn ease_in_out_cubic(t: f32) -> f32 {
            if t < 0.5 {
                4. * t * t * t
            } else {
                1. - (-2. * t + 2.).powi(3) / 2.
            }
        }

        // Sine
        pub fn ease_in_sine(t: f32) -> f32 {
            1. - f32::cos(t * std::f32::consts::FRAC_PI_2)
        }

        pub fn ease_out_sine(t: f32) -> f32 {
            f32::sin(t * std::f32::consts::FRAC_PI_2)
        }

        pub fn ease_in_out_sine(t: f32) -> f32 {
            -(f32::cos(std::f32::consts::PI * t) - 1.) / 2.
        }

        // Exponential
        pub fn ease_in_expo(t: f32) -> f32 {
            if t == 0. {
                0.
            } else {
                f32::powf(2., 10. * t - 10.)
            }
        }

        pub fn ease_out_expo(t: f32) -> f32 {
            if t == 1. {
                1.
            } else {
                1. - f32::powf(2., -10. * t)
            }
        }

        // Back (overshoot)
        pub fn ease_in_back(t: f32) -> f32 {
            let c1 = 1.70158;
            let c3 = c1 + 1.;
            c3 * t * t * t - c1 * t * t
        }

        pub fn ease_out_back(t: f32) -> f32 {
            let c1 = 1.70158;
            let c3 = c1 + 1.;
            1. + c3 * (t - 1.).powi(3) + c1 * (t - 1.).powi(2)
        }

        // Elastic
        pub fn ease_out_elastic(t: f32) -> f32 {
            if t == 0. {
                0.
            } else if t == 1. {
                1.
            } else {
                let c4 = (2. * std::f32::consts::PI) / 3.;
                f32::powf(2., -10. * t) * f32::sin((t * 10. - 0.75) * c4) + 1.
            }
        }

        // Bounce
        pub fn ease_out_bounce(t: f32) -> f32 {
            let n1 = 7.5625;
            let d1 = 2.75;

            if t < 1. / d1 {
                n1 * t * t
            } else if t < 2. / d1 {
                let t = t - 1.5 / d1;
                n1 * t * t + 0.75
            } else if t < 2.5 / d1 {
                let t = t - 2.25 / d1;
                n1 * t * t + 0.9375
            } else {
                let t = t - 2.625 / d1;
                n1 * t * t + 0.984375
            }
        }
    }

    pub mod f64 {
        // Linear
        pub fn linear(t: f64) -> f64 {
            t
        }

        pub fn smooth_step(t: f64) -> f64 {
            t * t * (3. - 2. * t)
        }

        pub fn smoother_step(t: f64) -> f64 {
            t * t * t * (t * (6. * t - 15.) + 10.)
        }

        // Quadratic
        pub fn ease_in_quad(t: f64) -> f64 {
            t * t
        }

        pub fn ease_out_quad(t: f64) -> f64 {
            1. - (1. - t) * (1. - t)
        }

        pub fn ease_in_out_quad(t: f64) -> f64 {
            if t < 0.5 {
                2. * t * t
            } else {
                1. - (-2. * t + 2.).powi(2) / 2.
            }
        }

        // Cubic
        pub fn ease_in_cubic(t: f64) -> f64 {
            t * t * t
        }

        pub fn ease_out_cubic(t: f64) -> f64 {
            1. - (1. - t).powi(3)
        }

        pub fn ease_in_out_cubic(t: f64) -> f64 {
            if t < 0.5 {
                4. * t * t * t
            } else {
                1. - (-2. * t + 2.).powi(3) / 2.
            }
        }

        // Sine
        pub fn ease_in_sine(t: f64) -> f64 {
            1. - f64::cos(t * std::f64::consts::FRAC_PI_2)
        }

        pub fn ease_out_sine(t: f64) -> f64 {
            f64::sin(t * std::f64::consts::FRAC_PI_2)
        }

        pub fn ease_in_out_sine(t: f64) -> f64 {
            -(f64::cos(std::f64::consts::PI * t) - 1.) / 2.
        }

        // Exponential
        pub fn ease_in_expo(t: f64) -> f64 {
            if t == 0. {
                0.
            } else {
                f64::powf(2., 10. * t - 10.)
            }
        }

        pub fn ease_out_expo(t: f64) -> f64 {
            if t == 1. {
                1.
            } else {
                1. - f64::powf(2., -10. * t)
            }
        }

        // Back (overshoot)
        pub fn ease_in_back(t: f64) -> f64 {
            let c1 = 1.70158;
            let c3 = c1 + 1.;
            c3 * t * t * t - c1 * t * t
        }

        pub fn ease_out_back(t: f64) -> f64 {
            let c1 = 1.70158;
            let c3 = c1 + 1.;
            1. + c3 * (t - 1.).powi(3) + c1 * (t - 1.).powi(2)
        }

        // Elastic
        pub fn ease_out_elastic(t: f64) -> f64 {
            if t == 0. {
                0.
            } else if t == 1. {
                1.
            } else {
                let c4 = (2. * std::f64::consts::PI) / 3.;
                f64::powf(2., -10. * t) * f64::sin((t * 10. - 0.75) * c4) + 1.
            }
        }

        // Bounce
        pub fn ease_out_bounce(t: f64) -> f64 {
            let n1 = 7.5625;
            let d1 = 2.75;

            if t < 1. / d1 {
                n1 * t * t
            } else if t < 2. / d1 {
                let t = t - 1.5 / d1;
                n1 * t * t + 0.75
            } else if t < 2.5 / d1 {
                let t = t - 2.25 / d1;
                n1 * t * t + 0.9375
            } else {
                let t = t - 2.625 / d1;
                n1 * t * t + 0.984375
            }
        }
    }
}

pub mod decay_curves {
    pub mod f32 {
        // Slower decay
        pub fn slow(t: f32) -> f32 {
            1. - f32::powf(0.7, t)
        }

        // Default
        pub fn default(t: f32) -> f32 {
            1. - f32::powf(0.5, t)
        }

        // Faster decay
        pub fn fast(t: f32) -> f32 {
            1. - f32::powf(0.2, t)
        }

        // Very snappy
        pub fn snappy(t: f32) -> f32 {
            1. - f32::powf(0.05, t)
        }
    }

    pub mod f64 {
        // Slower decay
        pub fn slow(t: f64) -> f64 {
            1. - f64::powf(0.7, t)
        }

        // Default
        pub fn default(t: f64) -> f64 {
            1. - f64::powf(0.5, t)
        }

        // Faster decay
        pub fn fast(t: f64) -> f64 {
            1. - f64::powf(0.2, t)
        }

        // Very snappy
        pub fn snappy(t: f64) -> f64 {
            1. - f64::powf(0.05, t)
        }
    }
}
