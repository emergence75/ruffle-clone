use crate::context::RenderContext;
use gc_arena::Collect;
use ruffle_render::backend::{RenderBackend, ShapeHandle};
use ruffle_render::bitmap::{BitmapHandle, BitmapInfo, BitmapSize, BitmapSource};
use ruffle_render::commands::CommandHandler;
use ruffle_render::matrix::Matrix;
use ruffle_render::shape_utils::{
    DistilledShape, DrawCommand, FillPath, FillStyle, LineStyle, ShapeFills, ShapeStrokes,
    StrokePath,
};
use ruffle_render::transform::Transform;
use std::cell::{Cell, RefCell};
use swf::{Rectangle, Twips};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct Drawing {
    fills_handle: Cell<Option<ShapeHandle>>,
    strokes_handle: Cell<Option<ShapeHandle>>,
    shape_strokes: RefCell<Option<ShapeStrokes>>,
    last_scale: Cell<(f32, f32)>,
    shape_bounds: Rectangle<Twips>,
    edge_bounds: Rectangle<Twips>,
    dirty: Cell<bool>,
    paths: Vec<DrawingPath>,
    bitmaps: Vec<BitmapInfo>,
    current_fill: Option<DrawingFill>,
    current_line: Option<DrawingLine>,
    pending_lines: Vec<DrawingLine>,
    cursor: (Twips, Twips),
    fill_start: (Twips, Twips),
}

impl Default for Drawing {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawing {
    pub fn new() -> Self {
        Self {
            fills_handle: Cell::new(None),
            strokes_handle: Cell::new(None),
            shape_strokes: RefCell::new(None),
            last_scale: Cell::new((0.0, 0.0)),
            shape_bounds: Default::default(),
            edge_bounds: Default::default(),
            dirty: Cell::new(false),
            paths: Vec::new(),
            bitmaps: Vec::new(),
            current_fill: None,
            current_line: None,
            pending_lines: Vec::new(),
            cursor: (Twips::ZERO, Twips::ZERO),
            fill_start: (Twips::ZERO, Twips::ZERO),
        }
    }

    pub fn set_fill_style(&mut self, style: Option<FillStyle>) {
        self.close_path();
        if let Some(existing) = self.current_fill.take() {
            self.paths.push(DrawingPath::Fill(existing));
        }
        self.paths
            .extend(self.pending_lines.drain(..).map(DrawingPath::Line));
        if let Some(mut existing) = self.current_line.take() {
            existing.is_closed = self.cursor == self.fill_start;
            let style = existing.style.clone();
            self.paths.push(DrawingPath::Line(existing));
            self.current_line = Some(DrawingLine {
                style,
                commands: vec![DrawCommand::MoveTo {
                    x: self.cursor.0,
                    y: self.cursor.1,
                }],
                is_closed: false,
            });
        }
        if let Some(style) = style {
            self.current_fill = Some(DrawingFill {
                style,
                commands: vec![DrawCommand::MoveTo {
                    x: self.cursor.0,
                    y: self.cursor.1,
                }],
            });
        }
        self.fill_start = self.cursor;
        self.dirty.set(true);
    }

    pub fn clear(&mut self) {
        self.current_fill = None;
        self.current_line = None;
        self.pending_lines.clear();
        self.paths.clear();
        self.bitmaps.clear();
        self.edge_bounds = Default::default();
        self.shape_bounds = Default::default();
        self.dirty.set(true);
        self.cursor = (Twips::ZERO, Twips::ZERO);
        self.fill_start = (Twips::ZERO, Twips::ZERO);
    }

    pub fn set_line_style(&mut self, style: Option<LineStyle>) {
        if let Some(mut existing) = self.current_line.take() {
            existing.is_closed = self.cursor == self.fill_start;
            if self.current_fill.is_some() {
                self.pending_lines.push(existing);
            } else {
                self.paths.push(DrawingPath::Line(existing));
            }
        }
        if let Some(style) = style {
            self.current_line = Some(DrawingLine {
                style,
                commands: vec![DrawCommand::MoveTo {
                    x: self.cursor.0,
                    y: self.cursor.1,
                }],
                is_closed: false,
            });
        }

        self.dirty.set(true);
    }

    pub fn draw_command(&mut self, command: DrawCommand) {
        let add_to_bounds = if let DrawCommand::MoveTo { x, y } = command {
            // Close any pending fills before moving.
            self.close_path();
            self.fill_start = (x, y);
            false
        } else {
            true
        };

        // Add command to current fill.
        if let Some(fill) = &mut self.current_fill {
            fill.commands.push(command.clone());
        }
        // Add command to current line.
        let stroke_width = if let Some(line) = &mut self.current_line {
            line.commands.push(command.clone());
            line.style.width()
        } else {
            Twips::ZERO
        };

        // Expand bounds.
        if add_to_bounds {
            if self.fill_start == self.cursor {
                // If this is the initial command after a move, include the starting point.
                let command = DrawCommand::MoveTo {
                    x: self.cursor.0,
                    y: self.cursor.1,
                };
                self.shape_bounds = stretch_bounds(&self.shape_bounds, &command, stroke_width);
                self.edge_bounds = stretch_bounds(&self.edge_bounds, &command, Twips::ZERO);
            }
            self.shape_bounds = stretch_bounds(&self.shape_bounds, &command, stroke_width);
            self.edge_bounds = stretch_bounds(&self.edge_bounds, &command, Twips::ZERO);
        }

        self.cursor = command.end_point();
        self.dirty.set(true);
    }

    pub fn add_bitmap(&mut self, bitmap: BitmapInfo) -> u16 {
        let id = self.bitmaps.len() as u16;
        self.bitmaps.push(bitmap);
        id
    }

    pub fn render(&self, context: &mut RenderContext) {
        if self.dirty.get() {
            self.dirty.set(false);
            let mut fills = Vec::with_capacity(self.paths.len());
            let mut strokes = Vec::with_capacity(self.paths.len());

            for path in &self.paths {
                match path {
                    DrawingPath::Fill(fill) => {
                        fills.push(FillPath {
                            style: fill.style.to_owned(),
                            commands: fill.commands.to_owned(),
                        });
                    }
                    DrawingPath::Line(line) => {
                        strokes.push(StrokePath {
                            style: line.style.to_owned(),
                            commands: line.commands.to_owned(),
                            is_closed: line.is_closed,
                        });
                    }
                }
            }

            if let Some(fill) = &self.current_fill {
                fills.push(FillPath {
                    style: fill.style.to_owned(),
                    commands: fill.commands.to_owned(),
                })
            }

            for line in &self.pending_lines {
                let mut commands = line.commands.to_owned();
                let is_closed = if self.current_fill.is_some() {
                    commands.push(DrawCommand::LineTo {
                        x: self.fill_start.0,
                        y: self.fill_start.1,
                    });
                    true
                } else {
                    self.cursor == self.fill_start
                };
                strokes.push(StrokePath {
                    style: line.style.to_owned(),
                    commands,
                    is_closed,
                })
            }

            if let Some(line) = &self.current_line {
                let mut commands = line.commands.to_owned();
                let is_closed = if self.current_fill.is_some() {
                    commands.push(DrawCommand::LineTo {
                        x: self.fill_start.0,
                        y: self.fill_start.1,
                    });
                    true
                } else {
                    self.cursor == self.fill_start
                };
                strokes.push(StrokePath {
                    style: line.style.to_owned(),
                    commands,
                    is_closed,
                })
            }

            let shape = DistilledShape {
                fills: ShapeFills {
                    paths: fills,
                    bounds: self.shape_bounds.clone(),
                },
                strokes: ShapeStrokes {
                    paths: strokes,
                    bounds: self.edge_bounds.clone(),
                },
                id: 0,
            };
            if let Some(handle) = self.fills_handle.get() {
                context
                    .renderer
                    .replace_shape_fills(&shape.fills, 0, handle);
            } else {
                self.fills_handle
                    .set(Some(context.renderer.register_shape_fills(&shape.fills, 0)));
            }
            *self.shape_strokes.borrow_mut() = Some(shape.strokes);
            self.last_scale.set((0.0, 0.0)); // Force recreation of stroke
        }

        if let Some(handle) = self.fills_handle.get() {
            context
                .commands
                .render_shape(handle, context.transform_stack.transform(), false);
        }

        // Update the stroke if we're drawing it at a different scale than last time
        let old_scale = self.last_scale.get();
        let cur_matrix = context.transform_stack.transform().matrix;
        let render_stroke_matrix = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: cur_matrix.tx,
            ty: cur_matrix.ty,
        };
        let cur_scale = (
            f32::abs(cur_matrix.a + cur_matrix.c),
            f32::abs(cur_matrix.b + cur_matrix.d),
        );
        if old_scale != cur_scale {
            let build_stroke_matrix = Matrix {
                a: cur_matrix.a,
                b: cur_matrix.b,
                c: cur_matrix.c,
                d: cur_matrix.d,
                tx: Default::default(),
                ty: Default::default(),
            };
            let strokes = self.shape_strokes.borrow();
            if let Some(strokes) = strokes.as_ref() {
                if let Some(handle) = self.strokes_handle.get() {
                    context
                        .renderer
                        .replace_shape_strokes(strokes, 0, build_stroke_matrix, handle);
                } else {
                    self.strokes_handle
                        .set(Some(context.renderer.register_shape_strokes(
                            strokes,
                            0,
                            build_stroke_matrix,
                        )));
                }
            }
            self.last_scale.set(cur_scale);
        }

        if let Some(render_handle) = self.strokes_handle.get() {
            context.commands.render_shape(
                render_handle,
                Transform {
                    matrix: render_stroke_matrix,
                    color_transform: context.transform_stack.transform().color_transform,
                },
                true,
            );
        }
    }

    pub fn self_bounds(&self) -> &Rectangle<Twips> {
        &self.shape_bounds
    }

    pub fn hit_test(
        &self,
        point: (Twips, Twips),
        local_matrix: &ruffle_render::matrix::Matrix,
    ) -> bool {
        use ruffle_render::shape_utils;
        for path in &self.paths {
            match path {
                DrawingPath::Fill(fill) => {
                    if shape_utils::draw_command_fill_hit_test(&fill.commands, point) {
                        return true;
                    }
                }
                DrawingPath::Line(line) => {
                    if shape_utils::draw_command_stroke_hit_test(
                        &line.commands,
                        line.style.width(),
                        point,
                        local_matrix,
                    ) {
                        return true;
                    }
                }
            }
        }

        // The pending fill will auto-close.
        if let Some(fill) = &self.current_fill {
            if shape_utils::draw_command_fill_hit_test(&fill.commands, point) {
                return true;
            }
        }

        for line in &self.pending_lines {
            if shape_utils::draw_command_stroke_hit_test(
                &line.commands,
                line.style.width(),
                point,
                local_matrix,
            ) {
                return true;
            }
        }

        if let Some(line) = &self.current_line {
            if shape_utils::draw_command_stroke_hit_test(
                &line.commands,
                line.style.width(),
                point,
                local_matrix,
            ) {
                return true;
            }

            // Stroke auto-closes if part of a fill; also check the closing line segment.
            if self.current_fill.is_some()
                && self.cursor != self.fill_start
                && shape_utils::draw_command_stroke_hit_test(
                    &[
                        DrawCommand::MoveTo {
                            x: self.cursor.0,
                            y: self.cursor.1,
                        },
                        DrawCommand::LineTo {
                            x: self.fill_start.0,
                            y: self.fill_start.1,
                        },
                    ],
                    line.style.width(),
                    point,
                    local_matrix,
                )
            {
                return true;
            }
        }

        false
    }

    // Ensures that the path is closed for a pending fill.
    fn close_path(&mut self) {
        if let Some(fill) = &mut self.current_fill {
            if self.cursor != self.fill_start {
                fill.commands.push(DrawCommand::LineTo {
                    x: self.fill_start.0,
                    y: self.fill_start.1,
                });

                if let Some(line) = &mut self.current_line {
                    line.commands.push(DrawCommand::LineTo {
                        x: self.fill_start.0,
                        y: self.fill_start.1,
                    });
                }
                self.dirty.set(true);
            }
        }
    }
}

impl BitmapSource for Drawing {
    fn bitmap_size(&self, id: u16) -> Option<BitmapSize> {
        self.bitmaps.get(id as usize).map(|bm| BitmapSize {
            width: bm.width,
            height: bm.height,
        })
    }
    fn bitmap_handle(&self, id: u16, _backend: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        self.bitmaps.get(id as usize).map(|bm| bm.handle.clone())
    }
}

#[derive(Debug, Clone)]
struct DrawingFill {
    style: FillStyle,
    commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone)]
struct DrawingLine {
    style: LineStyle,
    commands: Vec<DrawCommand>,
    is_closed: bool,
}

#[derive(Debug, Clone)]
enum DrawingPath {
    Fill(DrawingFill),
    Line(DrawingLine),
}

fn stretch_bounds(
    bounds: &Rectangle<Twips>,
    command: &DrawCommand,
    stroke_width: Twips,
) -> Rectangle<Twips> {
    let radius = stroke_width / 2;
    let bounds = bounds.clone();
    match *command {
        DrawCommand::MoveTo { x, y } => bounds
            .encompass(x - radius, y - radius)
            .encompass(x + radius, y + radius),
        DrawCommand::LineTo { x, y } => bounds
            .encompass(x - radius, y - radius)
            .encompass(x + radius, y + radius),
        DrawCommand::CurveTo { x1, y1, x2, y2 } => bounds
            .encompass(x1 - radius, y1 - radius)
            .encompass(x1 + radius, y1 + radius)
            .encompass(x2 - radius, y2 - radius)
            .encompass(x2 + radius, y2 + radius),
    }
}
