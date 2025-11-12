use std::ops::{Add, Mul};

use glam::{Vec2, Vec4};

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum LayoutDirection {
    #[default]
    LTR,
    RTL,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Size {
    pub width: SizeConstraint,
    pub height: SizeConstraint,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SizeConstraint {
    Fill(f32),
    #[default]
    Wrap,
    Fixed(f32),
}

impl From<f32> for SizeConstraint {
    fn from(value: f32) -> Self {
        SizeConstraint::Fixed(value)
    }
}

impl From<f64> for SizeConstraint {
    fn from(value: f64) -> Self {
        SizeConstraint::Fixed(value as f32)
    }
}

impl Size {
    pub fn new(width: SizeConstraint, height: SizeConstraint) -> Self {
        Self { width, height }
    }

    pub fn fixed(width: f32, height: f32) -> Self {
        Self {
            width: SizeConstraint::Fixed(width),
            height: SizeConstraint::Fixed(height),
        }
    }

    pub fn fill() -> Self {
        Self {
            width: SizeConstraint::Fill(1.0),
            height: SizeConstraint::Fill(1.0),
        }
    }

    pub fn wrap() -> Self {
        Self {
            width: SizeConstraint::Wrap,
            height: SizeConstraint::Wrap,
        }
    }

    pub fn square(size: impl Into<SizeConstraint>) -> Self {
        let constraint = size.into();

        Self {
            width: constraint,
            height: constraint,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum AlignX {
    Left,
    Right,
    #[default]
    Start,
    End,
    Center,

    /// Custom fractional alignment.
    ///
    /// Range: -1.0 (left/start) to 1.0 (right/end), with 0.0 being center.
    Fraction(f32),
}

#[derive(Default, Debug, Clone, Copy)]
pub enum AlignY {
    #[default]
    Top,
    Bottom,
    Center,

    /// Custom fractional alignment.
    ///
    /// Range: -1.0 (top) to 1.0 (bottom), with 0.0 being center.
    Fraction(f32),
}

impl AlignX {
    #[inline]
    pub fn position(&self, layout_direction: LayoutDirection, boundary: f32, size: f32) -> f32 {
        match self {
            AlignX::Left => 0.,
            AlignX::Right => boundary - size,
            AlignX::Center => (boundary - size) / 2.,
            AlignX::Start => match layout_direction {
                LayoutDirection::LTR => 0.,
                LayoutDirection::RTL => boundary - size,
            },
            AlignX::End => match layout_direction {
                LayoutDirection::LTR => boundary - size,
                LayoutDirection::RTL => 0.,
            },
            AlignX::Fraction(fraction) => {
                // Convert -1.0..1.0 to 0.0..1.0
                let normalized = (fraction + 1.0) / 2.0;

                normalized * (boundary - size)
            }
        }
    }
}

impl AlignY {
    #[inline]
    pub fn position(&self, boundary: f32, size: f32) -> f32 {
        match self {
            AlignY::Top => 0.,
            AlignY::Bottom => boundary - size,
            AlignY::Center => (boundary - size) / 2.,
            AlignY::Fraction(fraction) => {
                // Convert -1.0..1.0 to 0.0..1.0
                let normalized = (fraction + 1.0) / 2.0;

                normalized * (boundary - size)
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum MainAxisAlignment {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CrossAxisAlignment {
    #[default]
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct EdgeInsets {
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

impl EdgeInsets {
    /// An [`EdgeInsets`] with all sides set to zero.
    pub const ZERO: Self = Self {
        top: 0.0,
        left: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// Creates a new [`EdgeInsets`] with all sides set to the same value.
    ///
    /// This is a convenience method for creating [`EdgeInsets`] when you want
    /// the same inset value for all sides.
    ///
    /// # Parameters
    ///
    /// * `value`: The f32 value to be used for all sides.
    ///
    /// # Returns
    ///
    /// Returns a new [`EdgeInsets`] instance with all sides set to `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use tech_paws_ui::EdgeInsets;
    ///
    /// let insets = EdgeInsets::all(10.0);
    /// assert_eq!(insets.top, 10.0);
    /// assert_eq!(insets.left, 10.0);
    /// assert_eq!(insets.right, 10.0);
    /// assert_eq!(insets.bottom, 10.0);
    /// ```
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            left: value,
            right: value,
            bottom: value,
        }
    }

    /// Creates a new [`EdgeInsets`] with symmetric horizontal and vertical insets.
    ///
    /// This method allows you to create an [`EdgeInsets`] instance where the left and right insets
    /// are the same (horizontal), and the top and bottom insets are the same (vertical).
    ///
    /// # Parameters
    ///
    /// * `horizontal`: The f32 value to be used for both left and right insets.
    /// * `vertical`: The f32 value to be used for both top and bottom insets.
    ///
    /// # Returns
    ///
    /// Returns a new [`EdgeInsets`] instance with the specified symmetric insets.
    ///
    /// # Example
    ///
    /// ```
    /// use tech_paws_ui::EdgeInsets;
    ///
    /// let insets = EdgeInsets::symmetric(20.0, 10.0);
    /// assert_eq!(insets.left, 20.0);
    /// assert_eq!(insets.right, 20.0);
    /// assert_eq!(insets.top, 10.0);
    /// assert_eq!(insets.bottom, 10.0);
    /// ```
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            left: horizontal,
            right: horizontal,
            bottom: vertical,
        }
    }

    /// Returns the sum of the left and right insets.
    ///
    /// This method is useful when you need to know the total horizontal inset.
    ///
    /// # Returns
    ///
    /// Returns the sum of `self.left` and `self.right` as an f32.
    ///
    /// # Example
    ///
    /// ```
    /// use tech_paws_ui::EdgeInsets;
    ///
    /// let insets = EdgeInsets {
    ///     top: 10.0,
    ///     left: 15.0,
    ///     right: 20.0,
    ///     bottom: 10.0
    /// };
    /// assert_eq!(insets.horizontal(), 35.0);
    /// ```
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Returns the sum of the top and bottom insets.
    ///
    /// This method is useful when you need to know the total vertical inset.
    ///
    /// # Returns
    ///
    /// Returns the sum of `self.top` and `self.bottom` as an f32.
    ///
    /// # Example
    ///
    /// ```
    /// use tech_paws_ui::EdgeInsets;
    ///
    /// let insets = EdgeInsets {
    ///     top: 15.0,
    ///     left: 10.0,
    ///     right: 10.0,
    ///     bottom: 25.0
    /// };
    /// assert_eq!(insets.vertical(), 40.0);
    /// ```
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Constraints {
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
}

impl Constraints {
    pub fn expand(&mut self, padding: EdgeInsets) {
        if let Some(value) = self.min_width {
            self.min_width = Some(value + padding.horizontal());
        }
        if let Some(value) = self.min_height {
            self.min_height = Some(value + padding.vertical());
        }
        if let Some(value) = self.max_width {
            self.max_width = Some(value + padding.horizontal());
        }
        if let Some(value) = self.max_height {
            self.max_height = Some(value + padding.vertical());
        }
    }

    pub fn exact_size(size: Size) -> Self {
        let width = match size.width {
            SizeConstraint::Fill(_) => None,
            SizeConstraint::Wrap => None,
            SizeConstraint::Fixed(value) => Some(value),
        };

        let height = match size.height {
            SizeConstraint::Fill(_) => None,
            SizeConstraint::Wrap => None,
            SizeConstraint::Fixed(value) => Some(value),
        };

        Constraints {
            min_width: width,
            min_height: width,
            max_width: height,
            max_height: height,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

impl PhysicalSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub(crate) fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

#[derive(Debug, Clone)]
pub struct View {
    pub size: PhysicalSize,
    pub scale_factor: f32,
    pub safe_area: EdgeInsets,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Mul<f32> for Rect {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

impl Rect {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(crate) fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            width: size.x,
            height: size.y,
        }
    }

    pub fn inverse_y(&self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            width: self.width,
            height: self.height,
        }
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub fn expand(&self, size: f32) -> Rect {
        Rect {
            x: self.x - size,
            y: self.y - size,
            width: self.width + size * 2.,
            height: self.height + size * 2.,
        }
    }

    pub fn shrink(&self, size: f32) -> Rect {
        self.expand(-size)
    }
}

pub(crate) fn point_with_rect_hit_test(point: Vec2, rect: Rect) -> bool {
    point.x >= rect.position().x
        && point.x <= rect.position().x + rect.size().x
        && point.y >= rect.position().y
        && point.y <= rect.position().y + rect.size().y
}

pub(crate) fn rect_contains_boundary(boundary: Rect, rect: Rect) -> bool {
    let left_top = boundary.position();
    let right_top = boundary.position() + Vec2::new(boundary.size().x, 0.);
    let left_bottom = boundary.position() + Vec2::new(0., boundary.size().y);
    let right_bottom = boundary.position() + Vec2::new(boundary.size().x, boundary.size().y);

    point_with_rect_hit_test(left_top, rect)
        || point_with_rect_hit_test(right_top, rect)
        || point_with_rect_hit_test(left_bottom, rect)
        || point_with_rect_hit_test(right_bottom, rect)
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub struct ColorOkLab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl ColorRgb {
    pub fn new(r: f32, b: f32, g: f32) -> Self {
        ColorRgb { r, g, b }
    }

    pub fn with_alpha(&self, a: f32) -> ColorRgba {
        ColorRgba::new(self.r, self.b, self.g, a)
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex & 0xFF0000) >> 16) as f32 / 255.,
            g: ((hex & 0x00FF00) >> 8) as f32 / 255.,
            b: (hex & 0x0000FF) as f32 / 255.,
        }
    }

    pub fn to_hex(&self) -> u32 {
        let r = (self.r * 255.) as u32;
        let g = (self.g * 255.) as u32;
        let b = (self.b * 255.) as u32;

        (r << 16) | (g << 8) | b
    }

    /// Source: https://bottosson.github.io/posts/oklab/
    pub fn to_oklab(&self) -> ColorOkLab {
        let r = self.r as f64;
        let g = self.g as f64;
        let b = self.b as f64;

        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        let l = l.cbrt();
        let m = m.cbrt();
        let s = s.cbrt();

        ColorOkLab {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }
}

impl ColorRgba {
    pub const TRANSPARENT: Self = Self {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };

    pub fn transparent() -> ColorRgba {
        ColorRgba::new(0., 0., 0., 0.)
    }

    pub fn new(r: f32, b: f32, g: f32, a: f32) -> Self {
        ColorRgba { r, g, b, a }
    }

    pub fn to_rgb(&self) -> ColorRgb {
        ColorRgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex & 0x00FF0000) >> 16) as f32 / 255.,
            g: ((hex & 0x0000FF00) >> 8) as f32 / 255.,
            b: (hex & 0x000000FF) as f32 / 255.,
            a: ((hex & 0xFF000000) >> 24) as f32 / 255.,
        }
    }

    pub fn to_hex(&self) -> u32 {
        let r = (self.r * 255.) as u32;
        let g = (self.g * 255.) as u32;
        let b = (self.b * 255.) as u32;
        let a = (self.a * 255.) as u32;

        (a << 24) | (r << 16) | (g << 8) | b
    }

    pub fn with_opacity(&self, opacity: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: opacity,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub struct Border {
    pub top: Option<BorderSide>,
    pub right: Option<BorderSide>,
    pub bottom: Option<BorderSide>,
    pub left: Option<BorderSide>,
}

#[derive(Default, Debug, Clone, PartialEq, Copy)]
pub struct BorderSide {
    pub width: f32,
    pub color: ColorRgba,
}

impl BorderRadius {
    /// Creates a BorderRadius with individual values for each corner
    pub fn new(top_left: f32, top_right: f32, bottom_left: f32, bottom_right: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    /// Creates a BorderRadius with the same radius for all corners
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    /// Creates a BorderRadius with the same radius for top and bottom
    pub fn vertical(top: f32, bottom: f32) -> Self {
        Self {
            top_left: top,
            top_right: top,
            bottom_left: bottom,
            bottom_right: bottom,
        }
    }

    /// Creates a BorderRadius with the same radius for left and right
    pub fn horizontal(left: f32, right: f32) -> Self {
        Self {
            top_left: left,
            top_right: right,
            bottom_left: left,
            bottom_right: right,
        }
    }

    /// Creates a BorderRadius with radius only on top corners
    pub fn top(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: 0.0,
            bottom_right: 0.0,
        }
    }

    /// Creates a BorderRadius with radius only on bottom corners
    pub fn bottom(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: 0.0,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    /// Creates a BorderRadius with radius only on left corners
    pub fn left(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: 0.0,
            bottom_left: radius,
            bottom_right: 0.0,
        }
    }

    /// Creates a BorderRadius with radius only on right corners
    pub fn right(radius: f32) -> Self {
        Self {
            top_left: 0.0,
            top_right: radius,
            bottom_left: 0.0,
            bottom_right: radius,
        }
    }
}

impl Border {
    /// Creates a Border with individual sides
    pub fn new(
        top: Option<BorderSide>,
        right: Option<BorderSide>,
        bottom: Option<BorderSide>,
        left: Option<BorderSide>,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Creates a Border with symmetric horizontal (left/right) and vertical (top/bottom) sides
    pub fn symmetric(horizontal: BorderSide, vertical: BorderSide) -> Self {
        Self {
            top: Some(vertical),
            right: Some(horizontal),
            bottom: Some(vertical),
            left: Some(horizontal),
        }
    }

    /// Creates a Border with the same side applied to all edges
    pub fn all(side: BorderSide) -> Self {
        Self {
            top: Some(side),
            right: Some(side),
            bottom: Some(side),
            left: Some(side),
        }
    }

    /// Creates a Border with only horizontal sides (left and right)
    pub fn horizontal(side: BorderSide) -> Self {
        Self {
            top: None,
            right: Some(side),
            bottom: None,
            left: Some(side),
        }
    }

    /// Creates a Border with only vertical sides (top and bottom)
    pub fn vertical(side: BorderSide) -> Self {
        Self {
            top: Some(side),
            right: None,
            bottom: Some(side),
            left: None,
        }
    }

    /// Creates a Border with only a top side
    pub fn top(side: BorderSide) -> Self {
        Self {
            top: Some(side),
            right: None,
            bottom: None,
            left: None,
        }
    }

    /// Creates a Border with only a bottom side
    pub fn bottom(side: BorderSide) -> Self {
        Self {
            top: None,
            right: None,
            bottom: Some(side),
            left: None,
        }
    }

    /// Creates a Border with only a left side
    pub fn left(side: BorderSide) -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: Some(side),
        }
    }

    /// Creates a Border with only a right side
    pub fn right(side: BorderSide) -> Self {
        Self {
            top: None,
            right: Some(side),
            bottom: None,
            left: None,
        }
    }
}

impl BorderSide {
    /// Creates a new BorderSide
    pub fn new(width: f32, color: ColorRgba) -> Self {
        Self { width, color }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    Linear(LinearGradient),
    Radial(RadialGradient),
    Sweep(SweepGradient),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// Start point (normalized 0.0 to 1.0)
    pub start: (f32, f32),
    /// End point (normalized 0.0 to 1.0)
    pub end: (f32, f32),
    /// Color stops
    pub stops: Vec<ColorStop>,
    /// How to handle colors outside the gradient range
    pub tile_mode: TileMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// Center point (normalized 0.0 to 1.0)
    pub center: (f32, f32),
    /// Radius (normalized, typically 0.0 to 1.0)
    pub radius: f32,
    /// Optional focal point for elliptical gradients
    pub focal: Option<(f32, f32)>,
    /// Optional focal radius
    pub focal_radius: Option<f32>,
    /// Color stops
    pub stops: Vec<ColorStop>,
    /// How to handle colors outside the gradient range
    pub tile_mode: TileMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SweepGradient {
    /// Center point (normalized 0.0 to 1.0)
    pub center: (f32, f32),
    /// Start angle in radians (0 = right, π/2 = down)
    pub start_angle: f32,
    /// End angle in radians
    pub end_angle: f32,
    /// Color stops
    pub stops: Vec<ColorStop>,
    /// How to handle colors outside the gradient range
    pub tile_mode: TileMode,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ColorStop {
    /// Position along the gradient (0.0 to 1.0)
    pub offset: f32,
    /// Color at this position
    pub color: ColorRgba,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TileMode {
    /// Clamp to edge colors
    Clamp,
    /// Repeat the gradient
    Repeat,
    /// Repeat the gradient in reverse (mirror)
    Mirror,
    /// Decal - render transparent outside gradient
    Decal,
}

impl LinearGradient {
    /// Creates a simple top-to-bottom gradient
    pub fn vertical(colors: Vec<ColorRgba>) -> Self {
        Self {
            start: (0.5, 0.0),
            end: (0.5, 1.0),
            stops: Self::even_stops(colors),
            tile_mode: TileMode::Clamp,
        }
    }

    /// Creates a simple left-to-right gradient
    pub fn horizontal(colors: Vec<ColorRgba>) -> Self {
        Self {
            start: (0.0, 0.5),
            end: (1.0, 0.5),
            stops: Self::even_stops(colors),
            tile_mode: TileMode::Clamp,
        }
    }

    /// Creates a gradient at a specific angle (in radians)
    pub fn angled(angle: f32, colors: Vec<ColorRgba>) -> Self {
        let (dx, dy) = (angle.cos(), angle.sin());
        Self {
            start: (0.5 - dx * 0.5, 0.5 - dy * 0.5),
            end: (0.5 + dx * 0.5, 0.5 + dy * 0.5),
            stops: Self::even_stops(colors),
            tile_mode: TileMode::Clamp,
        }
    }

    pub fn new(start: (f32, f32), end: (f32, f32), stops: Vec<ColorStop>) -> Self {
        Self {
            start,
            end,
            stops,
            tile_mode: TileMode::Clamp,
        }
    }

    fn even_stops(colors: Vec<ColorRgba>) -> Vec<ColorStop> {
        let count = colors.len();
        if count == 0 {
            return vec![];
        }
        colors
            .into_iter()
            .enumerate()
            .map(|(i, color)| ColorStop {
                offset: i as f32 / (count - 1).max(1) as f32,
                color,
            })
            .collect()
    }
}

impl RadialGradient {
    /// Creates a simple radial gradient from center
    pub fn circle(colors: Vec<ColorRgba>) -> Self {
        Self {
            center: (0.5, 0.5),
            radius: 0.5,
            focal: None,
            focal_radius: None,
            stops: LinearGradient::even_stops(colors),
            tile_mode: TileMode::Clamp,
        }
    }

    pub fn new(center: (f32, f32), radius: f32, stops: Vec<ColorStop>) -> Self {
        Self {
            center,
            radius,
            focal: None,
            focal_radius: None,
            stops,
            tile_mode: TileMode::Clamp,
        }
    }
}

impl SweepGradient {
    /// Creates a full 360° sweep gradient
    pub fn full(colors: Vec<ColorRgba>) -> Self {
        Self {
            center: (0.5, 0.5),
            start_angle: 0.0,
            end_angle: std::f32::consts::TAU, // 2π
            stops: LinearGradient::even_stops(colors),
            tile_mode: TileMode::Clamp,
        }
    }

    pub fn new(
        center: (f32, f32),
        start_angle: f32,
        end_angle: f32,
        stops: Vec<ColorStop>,
    ) -> Self {
        Self {
            center,
            start_angle,
            end_angle,
            stops,
            tile_mode: TileMode::Clamp,
        }
    }
}

impl ColorStop {
    pub fn new(offset: f32, color: ColorRgba) -> Self {
        Self { offset, color }
    }
}

impl From<Vec4> for ColorRgba {
    fn from(value: Vec4) -> Self {
        ColorRgba {
            r: value.x,
            g: value.y,
            b: value.z,
            a: value.w,
        }
    }
}

impl From<Vec4> for Rect {
    fn from(value: Vec4) -> Self {
        Rect {
            x: value.x,
            y: value.y,
            width: value.z,
            height: value.w,
        }
    }
}
