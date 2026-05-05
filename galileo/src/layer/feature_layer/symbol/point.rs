#[cfg(not(target_arch = "wasm32"))]
use std::ops::Deref;
use std::sync::Arc;

use galileo_types::MultiPoint;
use galileo_types::cartesian::{CartesianPoint3d, Point2, Point3, Vector2};
use galileo_types::geometry::Geom;
use galileo_types::impls::ClosedContour;
use image::EncodableLayout;

use crate::Color;
use crate::decoded_image::DecodedImage;
use crate::error::GalileoError;
use crate::layer::feature_layer::symbol::Symbol;
use crate::render::point_paint::{MarkerStyle, PointPaint};
use crate::render::render_bundle::RenderBundle;
use crate::view::MapView;

/// Renders a point as a circle of fixes size.
#[derive(Debug, Copy, Clone)]
pub struct CirclePointSymbol {
    /// Color of the circle.
    pub color: Color,
    /// Diameter of the circle.
    pub size: f64,
}

impl CirclePointSymbol {
    /// Create a new instance.
    pub fn new(color: Color, size: f64) -> Self {
        Self { color, size }
    }
}

impl<F> Symbol<F> for CirclePointSymbol {
    fn render(
        &self,
        _feature: &F,
        geometry: &Geom<Point3>,
        min_resolution: f64,
        bundle: &mut RenderBundle,
        view: &MapView,
    ) {
        let paint = PointPaint::circle(self.color, self.size as f32);
        match geometry {
            Geom::Point(point) => {
                bundle.add_point(point, &paint, min_resolution, view);
            }
            Geom::MultiPoint(points) => {
                points.iter_points().for_each(|p| {
                    bundle.add_point(&p, &paint, min_resolution, view);
                });
            }
            _ => {}
        }
    }
}

/// Renders a point as an outlined circle of fixed size.
#[derive(Debug, Copy, Clone)]
pub struct OutlinedCirclePointSymbol {
    /// Color of the circle.
    pub color: Color,
    /// Color of the outline.
    pub outline: Color,
    /// Diameter of the circle.
    pub size: f64,
    /// Diameter of the outline.
    pub thickness: f64,
}

impl OutlinedCirclePointSymbol {
    /// Create a new instance.
    pub fn new(inner: Color, outline: Color, size: f64, thickness: f64) -> Self {
        Self {
            color: inner,
            outline,
            size,
            thickness,
        }
    }
}

impl<F> Symbol<F> for OutlinedCirclePointSymbol {
    fn render(
        &self,
        _feature: &F,
        geometry: &Geom<Point3>,
        min_resolution: f64,
        bundle: &mut RenderBundle,
        view: &MapView,
    ) {
        let inner = PointPaint::circle(self.color, self.size as f32);
        let outer = PointPaint::circle(self.outline, (self.size + self.thickness) as f32);
        match geometry {
            Geom::Point(point) => {
                bundle.add_point(point, &outer, min_resolution, view);
                bundle.add_point(point, &inner, min_resolution, view);
            }
            Geom::MultiPoint(points) => {
                points.iter_points().for_each(|p| {
                    bundle.add_point(&p, &outer, min_resolution, view);
                    bundle.add_point(&p, &inner, min_resolution, view);
                });
            }
            _ => {}
        }
    }
}

/// Renders a point as an outlined circle of fixed size.
#[derive(Debug, Copy, Clone)]
pub struct ArrowPointSymbol {
    /// Color of the shape.
    pub color: Color,
    /// Diameter of the circle.
    pub size: f64,
    /// Width of the arrow
    pub width: f32,
    /// Length of the arrow
    pub length: f32,
}

impl<F> Symbol<F> for ArrowPointSymbol {
    fn render(
        &self,
        _feature: &F,
        geometry: &Geom<Point3>,
        min_resolution: f64,
        bundle: &mut RenderBundle,
        view: &MapView,
    ) {
        let circle = PointPaint::circle(self.color, self.size as f32);
        match geometry {
            &Geom::Point(point) => {
                self.add_arrow_shape(min_resolution, bundle, view, point, self.width, self.length);
                let point = Point3::new(point.x(), point.y(), 0.0);
                bundle.add_point(&point, &circle, min_resolution, view);
            }
            Geom::MultiPoint(points) => {
                points.iter_points().for_each(|point| {
                    self.add_arrow_shape(
                        min_resolution,
                        bundle,
                        view,
                        point,
                        self.width,
                        self.length,
                    );
                    let pt = Point3::new(point.x(), point.y(), 0.0);
                    bundle.add_point(&pt, &circle, min_resolution, view);
                });
            }
            // TODO: use contour or something to specify width and length
            _ => {}
        }
    }
}

impl ArrowPointSymbol {
    pub fn new(color: Color, size: f64, width: f32, length: f32) -> Self {
        Self {
            color,
            size,
            width,
            length,
        }
    }

    fn add_arrow_shape(
        &self,
        min_resolution: f64,
        bundle: &mut RenderBundle,
        view: &MapView,
        point: Point3,
        width: f32,
        length: f32,
    ) {
        let heading = point.z() as f32 * (std::f32::consts::PI / 180.);
        let point = Point3::new(point.x(), point.y(), 0.0);
        const HEAD_LEN: f32 = 0.5;
        const HEAD_WID: f32 = 0.33;
        let cntr = Point2::<f32>::new(0.0, 0.0);
        let perp = Vector2::<f32>::new(-width * heading.cos(), width * heading.sin());
        let dir = Vector2::<f32>::new(heading.sin() * length, heading.cos() * length);
        let tip = cntr + dir;
        let head = cntr + dir * HEAD_LEN;
        bundle.add_point(
            &point,
            &PointPaint::shape(
                self.color,
                &ClosedContour {
                    points: vec![
                        cntr,
                        (head + perp * HEAD_WID),
                        head + perp,
                        tip,
                        head - perp,
                        (head - perp * HEAD_WID),
                        cntr,
                    ],
                },
                1.,
                heading,
            ),
            min_resolution,
            view,
        );
    }
}

/// Symbol that renders a point with an image. The image size is fixed on the screen and does not depend on map
/// resolution.
pub struct ImagePointSymbol {
    image: Arc<DecodedImage>,
    offset: Vector2<f32>,
    scale: f32,
}

impl ImagePointSymbol {
    /// Loads the image from the file system path.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_path(path: &str, offset: Vector2<f32>, scale: f32) -> Result<Self, GalileoError> {
        use galileo_types::cartesian::Size;

        let image = image::ImageReader::open(path)?
            .decode()
            .map_err(|_| GalileoError::ImageDecode)?;

        Ok(Self {
            image: Arc::new(DecodedImage::from_raw(
                Vec::from(image.to_rgba8().deref()),
                Size::new(image.width(), image.height()),
            )?),
            offset,
            scale,
        })
    }

    /// Decodes the image from the raw bytes.
    pub fn from_bytes(data: &[u8], offset: Vector2<f32>, scale: f32) -> Result<Self, GalileoError> {
        use galileo_types::cartesian::Size;

        let image = image::load_from_memory(data)
            .map_err(|_| GalileoError::ImageDecode)?
            .to_rgba8();

        Ok(Self {
            image: Arc::new(DecodedImage::from_raw(
                Vec::from(image.as_bytes()),
                Size::new(image.width(), image.height()),
            )?),
            offset,
            scale,
        })
    }
}

impl<F> Symbol<F> for ImagePointSymbol {
    fn render(
        &self,
        _feature: &F,
        geometry: &Geom<Point3>,
        _min_resolution: f64,
        bundle: &mut RenderBundle,
        view: &MapView,
    ) {
        let add_marker = |point: &Point3, bundle: &mut RenderBundle, view: &MapView| {
            bundle.add_marker(
                point,
                &MarkerStyle::Image {
                    image: self.image.clone(),
                    anchor: self.offset,
                    size: Some((self.image.size().cast::<f32>() * self.scale).cast()),
                },
                view,
            );
        };

        match geometry {
            Geom::Point(point) => add_marker(point, bundle, view),
            Geom::MultiPoint(points) => points.iter_points().for_each(|point| {
                add_marker(&point, bundle, view);
            }),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_symbol_from_file() {
        let symbol = ImagePointSymbol::from_path(
            "examples/data/pin-yellow.png",
            Vector2::new(0.5, 1.0),
            1.0,
        )
        .unwrap();
        assert_eq!(symbol.image.width(), 62);
        assert_eq!(symbol.image.height(), 99);
        assert_eq!(symbol.image.byte_size(), 62 * 99 * 4);
    }
}
