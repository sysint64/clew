use glam::Vec2;

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
    pub width: f32,
    pub height: f32,
}

impl PhysicalSize {
    pub(crate) fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
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
