use crate::caribou::batch::{Batch, BatchOp, Brush, Material, Path, PathOp, TextAlign, Transform};
use crate::cb_backend_skia_gl::WrappedSkiaFont;

type SkiaCanvas = skia_safe::Canvas;
type SkiaPaint = skia_safe::Paint;
type SkiaPath = skia_safe::Path;
type SkiaImage = skia_safe::Image;
type SkiaFont = skia_safe::Font;
type SkiaRect = skia_safe::Rect;
type SkiaPoint = skia_safe::Point;
type SkiaColor4f = skia_safe::Color4f;
type SkiaPaintStyle = skia_safe::PaintStyle;

pub fn skia_render_batch(canvas: &mut SkiaCanvas, batch: Batch) {
    for op in batch.unwrap() {
        match op {
            BatchOp::Path { transform, path, brush } => {
                let checkpoint = canvas.save();
                skia_apply_transform(canvas, transform);
                let skia_path = skia_render_path(path);
                let (stroke, fill) = skia_brush_to_stroke_fill_paint(brush);
                canvas.draw_path(&skia_path, &fill);
                canvas.draw_path(&skia_path, &stroke);
                canvas.restore_to_count(checkpoint);
            }
            BatchOp::Image { transform, image } => {
                let checkpoint = canvas.save();
                skia_apply_transform(canvas, transform);
                todo!();
                canvas.restore_to_count(checkpoint);
            }
            BatchOp::Text(top) => {
                let checkpoint = canvas.save();
                skia_apply_transform(canvas, top.transform);
                let font = top.font.get::<WrappedSkiaFont>().unwrap();
                let (stroke, fill) =
                    skia_brush_to_stroke_fill_paint(top.brush);
                let (_, stroke_bounds) = font
                    .measure_str(top.text.as_str(), Some(&stroke));
                let (_, fill_bounds) = font
                    .measure_str(top.text.as_str(), Some(&fill));
                let mut bounds = stroke_bounds;
                bounds.join(&fill_bounds);
                let offset: SkiaPoint = match top.align {
                    TextAlign::Origin => SkiaPoint::default(),
                    TextAlign::Center => (-bounds.width() / 2.0,
                                          bounds.height() / 2.0).into(),
                };
                canvas.draw_str(top.text.as_str(),
                                offset,
                                &font, &fill);
                canvas.draw_str(top.text.as_str(),
                                offset,
                                &font, &stroke);
                canvas.restore_to_count(checkpoint);
            }
            BatchOp::Batch { transform, batch } => {
                let checkpoint = canvas.save();
                skia_apply_transform(canvas, transform);
                skia_render_batch(canvas, batch);
                canvas.restore_to_count(checkpoint);
            }
        }
    }
}

pub fn skia_apply_transform(canvas: &mut SkiaCanvas, transform: Transform) {
    canvas.translate((transform.translate.x,
                      transform.translate.y));
    canvas.scale((transform.scale.x,
                  transform.scale.y));
    canvas.rotate(transform.rotate,
                  Some((transform.rotate_center.x,
                        transform.rotate_center.y).into()));
}

pub fn skia_brush_to_stroke_fill_paint(brush: Brush) -> (SkiaPaint, SkiaPaint) {
    let mut stroke = skia_material_to_paint(brush.stroke);
    stroke.set_style(SkiaPaintStyle::Stroke);
    stroke.set_stroke_width(brush.width);
    let mut fill = skia_material_to_paint(brush.fill);
    fill.set_style(SkiaPaintStyle::Fill);
    (stroke, fill)
}

pub fn skia_material_to_paint(material: Material) -> SkiaPaint {
    let mut paint = match material {
        Material::Transparent =>
            SkiaPaint::new(
                SkiaColor4f::new(0.0, 0.0, 0.0, 0.0),
                None),
        Material::Solid(color) =>
            SkiaPaint::new(
                SkiaColor4f::new(color.r, color.g, color.b, color.a),
                None),
    };
    paint.set_anti_alias(true);
    paint
}

pub fn skia_render_path(path: Path) -> SkiaPath {
    let mut skia_path = SkiaPath::new();
    for op in path.unwrap() {
        match op {
            PathOp::MoveTo(p) => {
                skia_path.move_to((p.x, p.y));
            }
            PathOp::LineTo(p) => {
                skia_path.line_to((p.x, p.y));
            }
            PathOp::QuadTo(p1, p2) => {
                skia_path.quad_to((p1.x, p1.y), (p2.x, p2.y));

            }
            PathOp::CubicTo(p1, p2, p3) => {
                skia_path.cubic_to((p1.x, p1.y), (p2.x, p2.y), (p3.x, p3.y));
            }
            PathOp::Close => {
                skia_path.close();
            }
            PathOp::AddLine(p1, p2) => {
                skia_path.move_to((p1.x, p1.y));
                skia_path.line_to((p2.x, p2.y));
            }
            PathOp::AddRect(p1, dim) => {
                skia_path.add_rect(
                    SkiaRect::from_xywh(p1.x, p1.y, dim.x, dim.y), None);
            }
            PathOp::AddOval(p1, p2) => {
                skia_path.add_oval(
                    SkiaRect::from_xywh(p1.x, p1.y, p2.x, p2.y), None);
            }
        }
    }
    skia_path
}