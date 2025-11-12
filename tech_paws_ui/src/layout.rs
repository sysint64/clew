use crate::{
    AlignX, AlignY, Constraints, CrossAxisAlignment, EdgeInsets, LayoutDirection,
    MainAxisAlignment, Rect, Size, SizeConstraint, View, WidgetId, WidgetRef,
    rect_contains_boundary,
};
use glam::Vec2;

const RENDER_DEBUG_INFO: bool = false;
pub(crate) const RENDER_DEBUG_BOUNDARIES: bool = false;

#[derive(Debug)]
pub(crate) struct WidgetPlacement {
    pub(crate) widget_ref: WidgetRef,
    pub(crate) zindex: i32,
    pub(crate) boundary: Rect,
    pub(crate) rect: Rect,
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutCommand {
    BeginContainer {
        kind: ContainerKind,
        constraints: Constraints,
        size: Size,
    },
    EndContainer,
    BeginAlign {
        align_x: AlignX,
        align_y: AlignY,
    },
    EndAlign,
    Fixed {
        widget_ref: WidgetRef,
        constraints: Constraints,
        size: Size,
        zindex: i32,
    },
    Wrap {
        widget_ref: WidgetRef,
        constraints: Constraints,
        size: Size,
        zindex: i32,
        wrap: WrapKind,
    },
    Spacer {
        constraints: Constraints,
        size: Size,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum WrapKind {
    TextTruncateLine,
    TextWrapLine,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ContainerKind {
    VStack {
        spacing: f32,
        main_axis_alignment: MainAxisAlignment,
        cross_axis_alignment: CrossAxisAlignment,
    },
    HStack {
        spacing: f32,
        main_axis_alignment: MainAxisAlignment,
        cross_axis_alignment: CrossAxisAlignment,
        rtl_aware: bool,
    },
    Flow {
        spacing: f32,
        run_spacing: f32,
        rtl_aware: bool,
    },
    #[default]
    ZStack,
    Padding {
        padding: EdgeInsets,
    },
}

#[derive(Default, Debug, Clone, Copy)]
enum StackAxis {
    #[default]
    None,
    Horizontal {
        rtl_aware: bool,
        spacing: f32,
    },
    Vertical {
        spacing: f32,
    },
}

#[derive(Debug, Default, Clone, Copy)]
struct LayoutContainerCommand {
    kind: ContainerKind,
    constraints: Constraints,
    size: Size,
}

#[derive(Debug, Default, Clone, Copy)]
struct LayoutContainer {
    idx: usize,
    axis: StackAxis,
    command: LayoutContainerCommand,
}

#[derive(Default)]
pub(crate) struct LayoutState {
    cursor: usize,

    wrap_sizes: Vec<Vec2>,
    flex_sizes: Vec<Vec2>,
    actual_sizes: Vec<Vec2>,
    offsets: Vec<Vec2>,
    resizes: Vec<Vec2>,
    flex_x: Vec<f32>,
    flex_y: Vec<f32>,
    flex_sum_x: Vec<f32>,
    constraints: Vec<Constraints>,
    flex_sum_y: Vec<f32>,

    position_cursor: usize,
    positions: Vec<Vec2>,

    containers_stack_cursor: usize,
    layout_direction: LayoutDirection,
    parent_container: LayoutContainer,
    containers_stack: Vec<LayoutContainer>,

    align_stack_cursor: usize,
    align_x_stack: Vec<AlignX>,
    align_y_stack: Vec<AlignY>,
}

impl LayoutState {
    fn current_idx(&self) -> usize {
        self.cursor - 1
    }

    fn set_constraints(&mut self, constraints: Constraints) {
        self.constraints[self.cursor - 1] = constraints;
    }

    fn set_wrap_size(&mut self, value: Vec2) {
        self.wrap_sizes[self.cursor - 1] = value;
    }

    fn set_actual_size(&mut self, value: Vec2) {
        self.actual_sizes[self.cursor - 1] = value;
    }

    fn set_offset(&mut self, value: Vec2) {
        self.offsets[self.cursor - 1] = value;
    }

    fn set_resize(&mut self, value: Vec2) {
        self.resizes[self.cursor - 1] = value;
    }

    fn set_flex_x(&mut self, value: f32) {
        self.flex_x[self.cursor - 1] = value;
    }

    fn set_flex_y(&mut self, value: f32) {
        self.flex_y[self.cursor - 1] = value;
    }

    fn push_boundary(&mut self) {
        if self.wrap_sizes.len() <= self.cursor {
            self.wrap_sizes.push(Vec2::ZERO);
            self.flex_sizes.push(Vec2::ZERO);
            self.actual_sizes.push(Vec2::ZERO);
            self.offsets.push(Vec2::ZERO);
            self.resizes.push(Vec2::ZERO);
            self.flex_x.push(0.);
            self.flex_y.push(0.);
            self.flex_sum_x.push(0.);
            self.flex_sum_y.push(0.);
            self.constraints.push(Constraints::default());
        } else {
            self.wrap_sizes[self.cursor] = Vec2::ZERO;
            self.flex_sizes[self.cursor] = Vec2::ZERO;
            self.actual_sizes[self.cursor] = Vec2::ZERO;
            self.offsets[self.cursor] = Vec2::ZERO;
            self.resizes[self.cursor] = Vec2::ZERO;
            self.flex_x[self.cursor] = 0.;
            self.flex_y[self.cursor] = 0.;
            self.flex_sum_x[self.cursor] = 0.;
            self.flex_sum_y[self.cursor] = 0.;
            self.constraints[self.cursor] = Constraints::default();
        }

        self.cursor += 1;
    }

    fn push_position(&mut self, position: Vec2) {
        if self.positions.len() <= self.position_cursor {
            self.positions.push(position);
        } else {
            self.positions[self.position_cursor] = position;
        }

        self.position_cursor += 1;
    }

    fn pop_position(&mut self) -> Vec2 {
        self.position_cursor -= 1;

        self.positions[self.position_cursor]
    }

    fn push_align(&mut self, align_x: AlignX, align_y: AlignY) {
        if self.align_x_stack.len() <= self.align_stack_cursor {
            self.align_x_stack.push(align_x);
            self.align_y_stack.push(align_y);
        } else {
            self.align_x_stack[self.align_stack_cursor] = align_x;
            self.align_y_stack[self.align_stack_cursor] = align_y;
        }

        self.align_stack_cursor += 1;
    }

    fn pop_align(&mut self) -> (AlignX, AlignY) {
        self.align_stack_cursor -= 1;

        (
            self.align_x_stack[self.align_stack_cursor],
            self.align_y_stack[self.align_stack_cursor],
        )
    }

    fn get_align(&mut self) -> (AlignX, AlignY) {
        (
            self.align_x_stack[self.align_stack_cursor - 1],
            self.align_y_stack[self.align_stack_cursor - 1],
        )
    }

    fn push_container(&mut self, container: LayoutContainer) {
        if self.containers_stack.len() <= self.containers_stack_cursor {
            self.containers_stack.push(container);
        } else {
            self.containers_stack[self.containers_stack_cursor] = container;
        }

        self.containers_stack_cursor += 1;
    }

    fn pop_container(&mut self) -> LayoutContainer {
        self.containers_stack_cursor -= 1;

        self.containers_stack[self.containers_stack_cursor]
    }

    fn clear(&mut self) {
        self.parent_container = LayoutContainer {
            idx: 0,
            axis: StackAxis::None,
            command: Default::default(),
        };
        self.cursor = 0;
        self.position_cursor = 0;
        self.containers_stack_cursor = 0;
        self.align_stack_cursor = 0;
    }

    fn add_flex_sum(&mut self, size: Size) {
        self.add_flex_sum_x(size.width);
        self.add_flex_sum_y(size.height);
    }

    fn add_flex_sum_x(&mut self, width: SizeConstraint) {
        if let SizeConstraint::Fill(flex) = width {
            self.set_flex_x(flex);

            if let StackAxis::Horizontal { .. } = self.parent_container.axis {
                self.flex_sum_x[self.parent_container.idx] += flex;
            } else {
                self.flex_sum_x[self.parent_container.idx] = 1.;
            }
        }
    }

    fn add_flex_sum_y(&mut self, height: SizeConstraint) {
        if let SizeConstraint::Fill(flex) = height {
            self.set_flex_y(flex);

            if let StackAxis::Vertical { .. } = self.parent_container.axis {
                self.flex_sum_y[self.parent_container.idx] += flex;
            } else {
                self.flex_sum_y[self.parent_container.idx] = 1.;
            }
        }
    }

    fn add_container_size(&mut self, size: Size, wrap_size: Vec2) -> Vec2 {
        Vec2::new(
            self.add_width(size.width, wrap_size.x),
            self.add_height(size.height, wrap_size.y),
        )
    }

    fn add_size(&mut self, size: Size, constraints: Constraints, wrap_size: Vec2) -> Vec2 {
        let wrap_size = apply_constraints(wrap_size, constraints);

        let mut size = Vec2::new(
            self.add_width(size.width, wrap_size.x),
            self.add_height(size.height, wrap_size.y),
        );

        size = apply_constraints(size, constraints);

        self.set_wrap_size(wrap_size);
        self.set_actual_size(size);

        size
    }

    fn add_width(&mut self, width: SizeConstraint, wrap_width: f32) -> f32 {
        let wrap_size = self.wrap_sizes.get_mut(self.parent_container.idx).unwrap();
        let flex_sizes = self.flex_sizes.get_mut(self.parent_container.idx).unwrap();

        match width {
            SizeConstraint::Fixed(value) => {
                match self.parent_container.axis {
                    StackAxis::None => {
                        wrap_size.x = wrap_size.x.max(value);
                        flex_sizes.x = flex_sizes.x.max(value);
                    }
                    StackAxis::Horizontal { spacing, .. } => {
                        wrap_size.x += value + spacing;
                        flex_sizes.x += value + spacing;
                    }
                    StackAxis::Vertical { .. } => {
                        wrap_size.x = wrap_size.x.max(value);
                        flex_sizes.x = flex_sizes.x.max(value);
                    }
                };

                value
            }
            SizeConstraint::Wrap => {
                match self.parent_container.axis {
                    StackAxis::None => {
                        wrap_size.x = wrap_size.x.max(wrap_width);
                        flex_sizes.x = flex_sizes.x.max(wrap_width);
                    }
                    StackAxis::Horizontal { spacing, .. } => {
                        wrap_size.x += wrap_width + spacing;
                        flex_sizes.x += wrap_width + spacing;
                    }
                    StackAxis::Vertical { .. } => {
                        wrap_size.x = wrap_size.x.max(wrap_width);
                        flex_sizes.x = flex_sizes.x.max(wrap_width);
                    }
                };

                wrap_width
            }
            SizeConstraint::Fill(_) => {
                match self.parent_container.axis {
                    StackAxis::None => {
                        wrap_size.x = wrap_size.x.max(wrap_width);
                    }
                    StackAxis::Horizontal { spacing, .. } => {
                        wrap_size.x += wrap_width + spacing;
                        flex_sizes.x += spacing;
                    }
                    StackAxis::Vertical { .. } => {
                        wrap_size.x = wrap_size.x.max(wrap_width);
                    }
                };

                wrap_width
            }
        }
    }

    fn add_height(&mut self, height: SizeConstraint, wrap_height: f32) -> f32 {
        let wrap_size = self.wrap_sizes.get_mut(self.parent_container.idx).unwrap();
        let flex_sizes = self.flex_sizes.get_mut(self.parent_container.idx).unwrap();

        match height {
            SizeConstraint::Fixed(value) => {
                match self.parent_container.axis {
                    StackAxis::None => {
                        wrap_size.y = wrap_size.y.max(value);
                        flex_sizes.y = flex_sizes.y.max(value);
                    }
                    StackAxis::Horizontal { .. } => {
                        wrap_size.y = wrap_size.y.max(value);
                        flex_sizes.y = flex_sizes.y.max(value);
                    }
                    StackAxis::Vertical { spacing } => {
                        wrap_size.y += value + spacing;
                        flex_sizes.y += value + spacing;
                    }
                };

                value
            }
            SizeConstraint::Wrap => {
                match self.parent_container.axis {
                    StackAxis::None => {
                        wrap_size.y = wrap_size.y.max(wrap_height);
                        flex_sizes.y = flex_sizes.y.max(wrap_height);
                    }
                    StackAxis::Horizontal { .. } => {
                        wrap_size.y = wrap_size.y.max(wrap_height);
                        flex_sizes.y = flex_sizes.y.max(wrap_height);
                    }
                    StackAxis::Vertical { spacing } => {
                        wrap_size.y += wrap_height + spacing;
                        flex_sizes.y += wrap_height + spacing;
                    }
                };

                wrap_height
            }
            SizeConstraint::Fill(_) => {
                match self.parent_container.axis {
                    StackAxis::None => {
                        wrap_size.y = wrap_size.y.max(wrap_height);
                    }
                    StackAxis::Horizontal { .. } => {
                        wrap_size.y = wrap_size.y.max(wrap_height);
                    }
                    StackAxis::Vertical { spacing } => {
                        wrap_size.y += wrap_height + spacing;
                        flex_sizes.y += spacing;
                    }
                }

                wrap_height
            }
        }
    }
}

fn apply_constraints_width(width: f32, constraints: Constraints) -> f32 {
    let mut width = width;

    if let Some(min_width) = constraints.min_width {
        width = width.max(min_width);
    }
    if let Some(max_width) = constraints.max_width {
        width = width.min(max_width);
    }

    width
}

fn apply_constraints_height(height: f32, constraints: Constraints) -> f32 {
    let mut height = height;

    if let Some(min_height) = constraints.min_height {
        height = height.max(min_height);
    }
    if let Some(max_height) = constraints.max_height {
        height = height.min(max_height);
    }

    height
}

fn apply_constraints(size: Vec2, constraints: Constraints) -> Vec2 {
    let mut size = size;

    if let Some(min_width) = constraints.min_width {
        size.x = size.x.max(min_width);
    }
    if let Some(max_width) = constraints.max_width {
        size.x = size.x.min(max_width);
    }
    if let Some(min_height) = constraints.min_height {
        size.y = size.y.max(min_height);
    }
    if let Some(max_height) = constraints.max_height {
        size.y = size.y.min(max_height);
    }

    size
}

pub fn layout(
    layout_state: &mut LayoutState,
    view: &View,
    commands: &[LayoutCommand],
    widget_placements: &mut Vec<WidgetPlacement>,
) {
    layout_state.clear();

    // Pass 1 - Calculate fixed sizes and flex sum -------------------------------------------------
    // Root container
    layout_state.push_boundary();
    let view_size = view.size.to_vec2();
    let root_size = view_size / view.scale_factor;
    layout_state.actual_sizes[0] = root_size;

    for command in commands {
        match command {
            LayoutCommand::BeginContainer {
                kind,
                constraints,
                size,
            } => {
                layout_state.push_container(layout_state.parent_container);
                layout_state.push_boundary();
                layout_state.add_flex_sum(*size);
                layout_state.set_constraints(*constraints);

                match kind {
                    ContainerKind::VStack { spacing, .. } => {
                        layout_state.parent_container = LayoutContainer {
                            idx: layout_state.current_idx(),
                            axis: StackAxis::Vertical { spacing: *spacing },
                            command: LayoutContainerCommand {
                                kind: *kind,
                                constraints: *constraints,
                                size: *size,
                            },
                        };
                    }
                    ContainerKind::HStack {
                        spacing, rtl_aware, ..
                    } => {
                        layout_state.parent_container = LayoutContainer {
                            idx: layout_state.current_idx(),
                            axis: StackAxis::Horizontal {
                                spacing: *spacing,
                                rtl_aware: *rtl_aware,
                            },
                            command: LayoutContainerCommand {
                                kind: *kind,
                                constraints: *constraints,
                                size: *size,
                            },
                        };
                    }
                    ContainerKind::ZStack => {
                        layout_state.parent_container = LayoutContainer {
                            idx: layout_state.current_idx(),
                            axis: StackAxis::None,
                            command: LayoutContainerCommand {
                                kind: *kind,
                                constraints: *constraints,
                                size: *size,
                            },
                        };
                    }
                    ContainerKind::Padding { padding } => {
                        layout_state.set_offset(Vec2::new(padding.left, padding.top));
                        layout_state
                            .set_resize(Vec2::new(-padding.horizontal(), -padding.vertical()));

                        layout_state.parent_container = LayoutContainer {
                            idx: layout_state.current_idx(),
                            axis: StackAxis::None,
                            command: LayoutContainerCommand {
                                kind: *kind,
                                constraints: *constraints,
                                size: *size,
                            },
                        };
                    }
                    ContainerKind::Flow {
                        spacing, rtl_aware, ..
                    } => {
                        layout_state.parent_container = LayoutContainer {
                            idx: layout_state.current_idx(),
                            axis: StackAxis::Horizontal {
                                spacing: *spacing,
                                rtl_aware: *rtl_aware,
                            },
                            command: LayoutContainerCommand {
                                kind: *kind,
                                constraints: *constraints,
                                size: *size,
                            },
                        };
                    }
                }
            }
            LayoutCommand::EndContainer => {
                let wrap_size = layout_state
                    .wrap_sizes
                    .get_mut(layout_state.parent_container.idx)
                    .unwrap();

                let mut size = layout_state.parent_container.command.size;

                match layout_state.parent_container.command.kind {
                    ContainerKind::VStack { spacing, .. } => {
                        wrap_size.y -= spacing;
                        wrap_size.y = wrap_size.y.max(0.);
                    }
                    ContainerKind::HStack { spacing, .. } => {
                        wrap_size.x -= spacing;
                        wrap_size.x = wrap_size.x.max(0.);
                    }
                    ContainerKind::Flow { spacing, .. } => {
                        wrap_size.x -= spacing;
                        wrap_size.x = wrap_size.x.max(0.);
                        wrap_size.y -= spacing;
                        wrap_size.y = wrap_size.y.max(0.);
                    }
                    ContainerKind::ZStack => {}
                    ContainerKind::Padding { padding } => {
                        wrap_size.x += padding.horizontal();
                        wrap_size.y += padding.vertical();
                        wrap_size.x = wrap_size.x.max(0.);
                        wrap_size.y = wrap_size.y.max(0.);

                        size = Size {
                            width: match size.width {
                                SizeConstraint::Fill(_) => size.width,
                                _ => SizeConstraint::Wrap,
                            },
                            height: match size.height {
                                SizeConstraint::Fill(_) => size.height,
                                _ => SizeConstraint::Wrap,
                            },
                        };
                    }
                };

                let wrap_size = *wrap_size;
                let current_container_idx = layout_state.parent_container.idx;

                let constraints = layout_state.parent_container.command.constraints;
                layout_state.parent_container = layout_state.pop_container();

                let size = layout_state.add_container_size(size, wrap_size);
                layout_state.actual_sizes[current_container_idx] =
                    apply_constraints(size, constraints);
            }
            LayoutCommand::Fixed {
                constraints, size, ..
            } => {
                layout_state.push_boundary();
                layout_state.set_constraints(*constraints);
                layout_state.add_flex_sum_x(size.width);
                layout_state.add_flex_sum_y(size.height);
                layout_state.add_size(
                    *size,
                    *constraints,
                    Vec2::new(
                        constraints.min_width.unwrap_or(0.),
                        constraints.min_height.unwrap_or(0.),
                    ),
                );
            }
            LayoutCommand::Wrap { .. } => todo!(),
            LayoutCommand::Spacer { constraints, size } => {
                layout_state.push_boundary();
                layout_state.set_constraints(*constraints);
                layout_state.add_flex_sum(*size);
                layout_state.add_size(*size, *constraints, Vec2::ZERO);
            }
            LayoutCommand::BeginAlign { .. } | LayoutCommand::EndAlign => {
                // Nothing
            }
        }
    }

    debug_assert!(layout_state.containers_stack_cursor == 0);

    // Extra memory to simplify calculations
    layout_state.push_boundary();
    layout_state.flex_sum_x[layout_state.cursor - 1] = 0.;
    layout_state.flex_sum_y[layout_state.cursor - 1] = 0.;

    // Pass 2 - Widget placements ------------------------------------------------------------------
    let mut current_idx = 1; // Skip root container
    let mut current_position = Vec2::ZERO;

    widget_placements.clear();

    layout_state.push_position(current_position);
    layout_state.parent_container = LayoutContainer {
        idx: 0,
        axis: StackAxis::None,
        command: Default::default(),
    };
    layout_state.push_align(AlignX::Left, AlignY::Top);

    for command in commands {
        let mut go_next = true;
        let container_idx = layout_state.parent_container.idx;

        let container_resize = layout_state.resizes[container_idx];
        let container_offset = layout_state.offsets[container_idx];
        let container_size_resized = layout_state.actual_sizes[container_idx] + container_resize;
        let container_size = layout_state.actual_sizes[container_idx];

        let flex_x = layout_state.flex_x[current_idx];
        let flex_y = layout_state.flex_y[current_idx];

        if flex_x > 0. {
            let constraints = layout_state.constraints[current_idx];
            let container_flex_size = layout_state.flex_sizes[container_idx];

            let mut size = match layout_state.parent_container.axis {
                StackAxis::None => container_size_resized.x,
                StackAxis::Horizontal { spacing, .. } => {
                    let flex_sum_x = layout_state.flex_sum_x[container_idx].max(1.);
                    let available_width =
                        (container_size_resized.x - container_flex_size.x + spacing).max(0.);
                    let per_flex = available_width / flex_sum_x;

                    flex_x * per_flex
                }
                StackAxis::Vertical { .. } => container_size_resized.x,
            };

            size = apply_constraints_width(size, constraints);
            layout_state.actual_sizes[current_idx].x = size;
        }

        if flex_y > 0. {
            let constraints = layout_state.constraints[current_idx];
            let container_flex_size = layout_state.flex_sizes[container_idx];

            let mut size = match layout_state.parent_container.axis {
                StackAxis::None => container_size_resized.y,
                StackAxis::Horizontal { .. } => container_size_resized.y,
                StackAxis::Vertical { spacing } => {
                    let flex_sum_y = layout_state.flex_sum_y[container_idx].max(1.);
                    let available_height =
                        (container_size_resized.y - container_flex_size.y + spacing).max(0.);
                    let per_flex = available_height / flex_sum_y;

                    flex_y * per_flex
                }
            };

            size = apply_constraints_height(size, constraints);
            layout_state.actual_sizes[current_idx].y = size;
        }

        let mut widget_size = layout_state.actual_sizes[current_idx];

        let boundary_size = container_size;

        // Don't remember why I added this, it seems like it breaks flex size
        // ---------------------------------------------------------------------
        // let boundary_size = match layout_state.parent_container.axis {
        //     StackAxis::None => container_size,
        //     StackAxis::Horizontal { .. } => Vec2::new(widget_size.x, container_size.y),
        //     StackAxis::Vertical { .. } => Vec2::new(container_size.x, widget_size.y),
        // };
        // ---------------------------------------------------------------------

        let mut boundary = Rect::from_pos_size(current_position, boundary_size);
        let (align_x, align_y) = layout_state.get_align();
        let mut position = current_position
            + container_offset
            + Vec2::new(
                align_x.position(
                    layout_state.layout_direction,
                    boundary_size.x,
                    widget_size.x,
                ),
                align_y.position(boundary_size.y, widget_size.y),
            );

        if let StackAxis::Horizontal { rtl_aware, .. } = layout_state.parent_container.axis {
            if rtl_aware && layout_state.layout_direction == LayoutDirection::RTL {
                position.x -= widget_size.x;
                boundary.x -= widget_size.x;
            }
        }

        let rect = Rect::from_pos_size(position, widget_size);

        match command {
            LayoutCommand::BeginAlign { align_x, align_y } => {
                layout_state.push_align(*align_x, *align_y);
                continue;
            }
            LayoutCommand::EndAlign => {
                layout_state.pop_align();
                continue;
            }
            LayoutCommand::BeginContainer {
                kind,
                constraints,
                size,
            } => {
                layout_state.push_position(current_position);
                layout_state.push_container(layout_state.parent_container);
                current_position = position;
                let command = LayoutContainerCommand {
                    kind: *kind,
                    constraints: *constraints,
                    size: *size,
                };

                match kind {
                    ContainerKind::VStack { spacing, .. } => {
                        layout_state.parent_container = LayoutContainer {
                            idx: current_idx,
                            axis: StackAxis::Vertical { spacing: *spacing },
                            command,
                        };

                        current_idx += 1;
                        go_next = false;
                    }
                    ContainerKind::HStack {
                        spacing, rtl_aware, ..
                    } => {
                        if *rtl_aware && layout_state.layout_direction == LayoutDirection::RTL {
                            current_position = position + Vec2::new(widget_size.x, 0.);
                        }

                        layout_state.parent_container = LayoutContainer {
                            idx: current_idx,
                            axis: StackAxis::Horizontal {
                                spacing: *spacing,
                                rtl_aware: *rtl_aware,
                            },
                            command,
                        };

                        current_idx += 1;
                        go_next = false;
                    }
                    ContainerKind::Flow { .. } => todo!(),
                    ContainerKind::ZStack => {
                        layout_state.parent_container = LayoutContainer {
                            idx: current_idx,
                            axis: StackAxis::None,
                            command,
                        };

                        current_idx += 1;
                        go_next = false;
                    }
                    ContainerKind::Padding { .. } => {
                        layout_state.parent_container = LayoutContainer {
                            idx: current_idx,
                            axis: StackAxis::None,
                            command,
                        };

                        current_idx += 1;
                    }
                }
            }
            LayoutCommand::EndContainer => {
                widget_size = container_size;
                layout_state.parent_container = layout_state.pop_container();
                current_position = layout_state.pop_position();

                if RENDER_DEBUG_BOUNDARIES {
                    let container_idx = layout_state.parent_container.idx;
                    let container_offset = layout_state.offsets[container_idx];
                    let container_size = layout_state.actual_sizes[container_idx];
                    let position = current_position
                        + container_offset
                        + Vec2::new(
                            align_x.position(
                                layout_state.layout_direction,
                                container_size.x,
                                widget_size.x,
                            ),
                            align_y.position(container_size.y, widget_size.y),
                        );

                    // widget_placements.push(WidgetPlacement {
                    //     zindex: i32::MAX,
                    //     boundary: Rect::ZERO,
                    //     rect: Rect::from_pos_size(position, widget_size),
                    // });
                }
            }
            LayoutCommand::Fixed {
                widget_ref, zindex, ..
            }
            | LayoutCommand::Wrap {
                widget_ref, zindex, ..
            } => {
                // Don't render anything outside the screen view
                if rect_contains_boundary(boundary, Rect::from_pos_size(Vec2::ZERO, root_size)) {
                    widget_placements.push(WidgetPlacement {
                        widget_ref: *widget_ref,
                        zindex: *zindex,
                        boundary,
                        rect,
                    });
                }

                current_idx += 1;
            }
            LayoutCommand::Spacer { .. } => {
                current_idx += 1;
            }
        }

        if go_next {
            match layout_state.parent_container.axis {
                StackAxis::Horizontal { spacing, rtl_aware } => {
                    if rtl_aware && layout_state.layout_direction == LayoutDirection::RTL {
                        current_position.x -= widget_size.x + spacing
                    } else {
                        current_position.x += widget_size.x + spacing
                    }
                }
                StackAxis::Vertical { spacing, .. } => {
                    current_position.y += widget_size.y + spacing
                }
                StackAxis::None => {}
            }
        }
    }

    debug_assert!(layout_state.containers_stack_cursor == 0);
}
