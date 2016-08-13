//! Tools to build path objects from a sequence of imperative commands.

extern crate lyon_core;
extern crate lyon_bezier;
// TODO: remove sid;
extern crate sid;

mod arc;

use lyon_core::{ PrimitiveEvent, SvgEvent, ArcFlags };
use lyon_core::math::*;
use lyon_bezier::{ CubicBezierSegment, QuadraticBezierSegment };
use arc::arc_to_cubic_beziers;

#[derive(Debug)]
/// Phatom type marker for PathId.
pub struct Path_;
/// An Id that represents a sub-path in a certain path object.
pub type PathId = sid::Id<Path_, u16>;
pub fn path_id(idx: u16) -> PathId { PathId::new(idx) }


/// The base path building interface. More elaborate interfaces are built on top
/// of the provided primitives.
pub trait PrimitiveBuilder {
    type PathType;

    fn move_to(&mut self, to: Point);
    fn line_to(&mut self, to: Point);
    fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point);
    fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point);
    fn close(&mut self) -> PathId;
    fn current_position(&self) -> Point;

    fn primitive_event(&mut self, event: PrimitiveEvent) {
        match event {
            PrimitiveEvent::MoveTo(to) => { self.move_to(to); }
            PrimitiveEvent::LineTo(to) => { self.line_to(to); }
            PrimitiveEvent::QuadraticTo(ctrl, to) => { self.quadratic_bezier_to(ctrl, to); }
            PrimitiveEvent::CubicTo(ctrl1, ctrl2, to) => { self.cubic_bezier_to(ctrl1, ctrl2, to); }
            PrimitiveEvent::Close => { self.close(); }
        }
    }

    fn build(self) -> Self::PathType;
}

/// A path building interface that tries to stay close to SVG's path specification.
/// https://svgwg.org/specs/paths/
pub trait SvgBuilder : PrimitiveBuilder {
    fn relative_move_to(&mut self, to: Vec2);
    fn relative_line_to(&mut self, to: Vec2);
    fn relative_quadratic_bezier_to(&mut self, ctrl: Vec2, to: Vec2);
    fn relative_cubic_bezier_to(&mut self, ctrl1: Vec2, ctrl2: Vec2, to: Vec2);
    fn cubic_bezier_smooth_to(&mut self, ctrl2: Point, to: Point);
    fn relative_cubic_bezier_smooth_to(&mut self, ctrl2: Vec2, to: Vec2);
    fn quadratic_bezier_smooth_to(&mut self, to: Point);
    fn relative_quadratic_bezier_smooth_to(&mut self, to: Vec2);
    fn horizontal_line_to(&mut self, x: f32);
    fn relative_horizontal_line_to(&mut self, dx: f32);
    fn vertical_line_to(&mut self, y: f32);
    fn relative_vertical_line_to(&mut self, dy: f32);
    // TODO: Would it be better to use an api closer to cairo/skia for arcs?
    fn arc_to(&mut self, to: Point, radii: Vec2, x_rotation: f32, flags: ArcFlags);
    fn relative_arc_to(&mut self, to: Vec2, radii: Vec2, x_rotation: f32, flags: ArcFlags);

    fn svg_event(&mut self, event: SvgEvent) {
        match event {
            SvgEvent::MoveTo(to) => { self.move_to(to); }
            SvgEvent::LineTo(to) => { self.line_to(to); }
            SvgEvent::QuadraticTo(ctrl, to) => { self.quadratic_bezier_to(ctrl, to); }
            SvgEvent::CubicTo(ctrl1, ctrl2, to) => { self.cubic_bezier_to(ctrl1, ctrl2, to); }
            SvgEvent::Close => { self.close(); }

            SvgEvent::ArcTo(to, radii, x_rotation, flags) => { self.arc_to(to, radii, x_rotation, flags); }
            SvgEvent::RelativeArcTo(to, radii, x_rotation, flags) => { self.relative_arc_to(to, radii, x_rotation, flags); }

            SvgEvent::RelativeMoveTo(to) => { self.relative_move_to(to); }
            SvgEvent::RelativeLineTo(to) => { self.relative_line_to(to); }
            SvgEvent::RelativeQuadraticTo(ctrl, to) => { self.relative_quadratic_bezier_to(ctrl, to); }
            SvgEvent::RelativeCubicTo(ctrl1, ctrl2, to) => { self.relative_cubic_bezier_to(ctrl1, ctrl2, to); }

            SvgEvent::HorizontalLineTo(x) => { self.horizontal_line_to(x); }
            SvgEvent::VerticalLineTo(y) => { self.vertical_line_to(y); }
            SvgEvent::RelativeHorizontalLineTo(x) => { self.relative_horizontal_line_to(x); }
            SvgEvent::RelativeVerticalLineTo(y) => { self.relative_vertical_line_to(y); }
        }
    }
}

/// Build a path from a simple list of points.
pub trait PolygonBuilder {
    fn polygon(&mut self, points: &[Point]);
}

/// Implements the Svg building interface on top of the a primitive builder.
pub struct SvgPathBuilder<Builder: PrimitiveBuilder> {
    builder: Builder,
    last_ctrl: Point,
}

impl<Builder: PrimitiveBuilder> SvgPathBuilder<Builder> {
    pub fn from_builder(builder: Builder) -> SvgPathBuilder<Builder> {
        SvgPathBuilder {
            builder: builder,
            last_ctrl: vec2(0.0, 0.0),
        }
    }
}

impl<Builder: PrimitiveBuilder> PrimitiveBuilder for SvgPathBuilder<Builder> {
    type PathType = Builder::PathType;

    fn move_to(&mut self, to: Point) {
        self.last_ctrl = to;
        self.builder.move_to(to);
    }

    fn line_to(&mut self, to: Point) {
        self.last_ctrl = self.current_position();
        self.builder.line_to(to);
    }

    fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point) {
        self.last_ctrl = ctrl;
        self.builder.quadratic_bezier_to(ctrl, to);
    }

    fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.last_ctrl = ctrl2;
        self.builder.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    fn close(&mut self)  -> PathId {
        self.last_ctrl = point(0.0, 0.0);
        self.builder.close()
    }

    fn current_position(&self) -> Vec2 {
        self.builder.current_position()
    }

    fn build(self) -> Builder::PathType { self.builder.build() }
}

impl<Builder: PrimitiveBuilder> SvgBuilder for SvgPathBuilder<Builder> {
    fn relative_move_to(&mut self, to: Vec2) {
        let offset = self.builder.current_position();
        self.move_to(offset + to);
    }

    fn relative_line_to(&mut self, to: Vec2) {
        let offset = self.builder.current_position();
        self.line_to(offset + to);
    }

    fn relative_quadratic_bezier_to(&mut self, ctrl: Vec2, to: Vec2) {
        let offset = self.builder.current_position();
        self.quadratic_bezier_to(ctrl + offset, to + offset);
    }

    fn relative_cubic_bezier_to(&mut self, ctrl1: Vec2, ctrl2: Vec2, to: Vec2) {
        let offset = self.builder.current_position();
        self.cubic_bezier_to(ctrl1 + offset, ctrl2 + offset, to + offset);
    }

    fn cubic_bezier_smooth_to(&mut self, ctrl2: Point, to: Point) {
        let ctrl = self.builder.current_position() + (self.builder.current_position() - self.last_ctrl);
        self.cubic_bezier_to(ctrl, ctrl2, to);
    }

    fn relative_cubic_bezier_smooth_to(&mut self, ctrl2: Vec2, to: Vec2) {
        let ctrl = self.builder.current_position() - self.last_ctrl;
        self.relative_cubic_bezier_to(ctrl, ctrl2, to);
    }

    fn quadratic_bezier_smooth_to(&mut self, to: Point) {
        let ctrl = self.builder.current_position() + (self.builder.current_position() - self.last_ctrl);
        self.quadratic_bezier_to(ctrl, to);
    }

    fn relative_quadratic_bezier_smooth_to(&mut self, to: Vec2) {
        let ctrl = self.builder.current_position() - self.last_ctrl;
        self.relative_quadratic_bezier_to(ctrl, to);
    }

    fn horizontal_line_to(&mut self, x: f32) {
        let y = self.builder.current_position().y;
        self.line_to(vec2(x, y));
    }

    fn relative_horizontal_line_to(&mut self, dx: f32) {
        let p = self.builder.current_position();
        self.line_to(vec2(p.x + dx, p.y));
    }

    fn vertical_line_to(&mut self, y: f32) {
        let x = self.builder.current_position().x;
        self.line_to(vec2(x, y));
    }

    fn relative_vertical_line_to(&mut self, dy: f32) {
        let p = self.builder.current_position();
        self.line_to(vec2(p.x, p.y + dy));
    }

    // x_rotation in radian
    fn arc_to(&mut self, to: Point, radii: Vec2, x_rotation: f32, flags: ArcFlags) {

        // If end and starting point are identical, then there is not ellipse to be drawn
        if self.current_position() == to {
            return;
        }

        arc_to_cubic_beziers(
            self.current_position(),
            to, radii, x_rotation, flags,
            self
        );
    }

    fn relative_arc_to(&mut self, to: Vec2, radii: Vec2, x_rotation: f32, flags: ArcFlags) {
        let offset = self.builder.current_position();
        self.arc_to(to + offset, radii, x_rotation, flags);
    }
}

/// Generates flattened paths
pub struct FlattenedBuilder<Builder> {
    builder: Builder,
    tolerance: f32,
}

impl<Builder: PrimitiveBuilder> PrimitiveBuilder for FlattenedBuilder<Builder> {
    type PathType = Builder::PathType;

    fn move_to(&mut self, to: Point) { self.builder.move_to(to); }

    fn line_to(&mut self, to: Point) { self.builder.line_to(to); }

    fn quadratic_bezier_to(&mut self, ctrl: Point, to: Point) {
        QuadraticBezierSegment {
            from: self.current_position(),
            cp: ctrl,
            to: to
        }.flattened_for_each(self.tolerance, &mut |point| { self.line_to(point); });
    }

    fn cubic_bezier_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        CubicBezierSegment{
            from: self.current_position(),
            cp1: ctrl1,
            cp2: ctrl2,
            to: to,
        }.flattened_for_each(self.tolerance, &mut |point| { self.line_to(point); });
    }

    fn close(&mut self) -> PathId { self.builder.close() }

    fn current_position(&self) -> Point { self.builder.current_position() }

    fn build(self) -> Builder::PathType { self.builder.build() }
}

impl<Builder: PrimitiveBuilder> FlattenedBuilder<Builder> {
    pub fn new(builder: Builder, tolerance: f32) -> FlattenedBuilder<Builder> {
        FlattenedBuilder {
            builder: builder,
            tolerance: tolerance,
        }
    }

    pub fn set_tolerance(&mut self, tolerance: f32) { self.tolerance = tolerance }
}

impl<Builder: PrimitiveBuilder> PolygonBuilder for Builder {
    fn polygon(&mut self, points: &[Point]) {
        assert!(!points.is_empty());

        self.move_to(points[0]);
        for p in points[1..].iter() {
            self.line_to(*p);
        }
        self.close();
    }
}
