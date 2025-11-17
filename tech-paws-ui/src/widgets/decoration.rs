pub struct DecorationWidget;

pub struct DecorationBuilder<'a> {
    id: WidgetId,
    text: &'a str,
    width: SizeConstraint,
    constraints: Constraints,
    zindex: Option<i32>,
    color: ColorRgba,
    text_align_x: AlignX,
    text_align_y: AlignY,
}
