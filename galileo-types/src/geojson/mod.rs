use geojson::{GeometryValue, LineStringType, PolygonType, Position};

use crate::error::GalileoTypesError;
use crate::geo::impls::GeoPoint2d;
use crate::geo::{NewGeoPoint, Projection};
use crate::geometry::{Geom, Geometry};
use crate::impls::{Contour, MultiContour, MultiPoint, MultiPolygon, Polygon};

impl TryFrom<Position> for GeoPoint2d {
    type Error = GalileoTypesError;

    fn try_from(value: Position) -> Result<Self, Self::Error> {
        if value.len() < 2 {
            Err(GalileoTypesError::Conversion(
                "point must contain at least 2 dimensions".to_string(),
            ))
        } else {
            Ok(Self::latlon(value[1], value[0]))
        }
    }
}

impl Geometry for geojson::Geometry {
    type Point = GeoPoint2d;

    fn project<Proj>(&self, projection: &Proj) -> Option<Geom<Proj::OutPoint>>
    where
        Proj: Projection<InPoint = Self::Point> + ?Sized,
    {
        match &self.value {
            GeometryValue::Point { coordinates: p } => GeoPoint2d::try_from(p.clone()).ok()?.project(projection),
            GeometryValue::MultiPoint { coordinates: points } => convert_multi_point(points)?.project(projection),
            GeometryValue::LineString { coordinates: points } => convert_contour(points)?.project(projection),
            GeometryValue::MultiLineString { coordinates: lines } => convert_multi_contour(lines)?.project(projection),
            GeometryValue::Polygon { coordinates: polygon } => convert_polygon(polygon)?.project(projection),
            GeometryValue::MultiPolygon { coordinates: mp } => convert_multi_polygon(mp)?.project(projection),
            GeometryValue::GeometryCollection { geometries: _ } => todo!(),
        }
    }
}

fn convert_contour(line_string: &LineStringType) -> Option<Contour<GeoPoint2d>> {
    let is_closed = !line_string.is_empty() && line_string[0] == line_string[line_string.len() - 1];
    Some(Contour::new(
        line_string
            .iter()
            .map(|p| GeoPoint2d::try_from(p.clone()).ok())
            .collect::<Option<Vec<_>>>()?,
        is_closed,
    ))
}

fn convert_multi_point(points: &[Position]) -> Option<MultiPoint<GeoPoint2d>> {
    Some(MultiPoint::from(
        points
            .iter()
            .map(|p| GeoPoint2d::try_from(p.clone()).ok())
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn convert_multi_contour(lines: &[LineStringType]) -> Option<MultiContour<GeoPoint2d>> {
    Some(MultiContour::from(
        lines
            .iter()
            .map(convert_contour)
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn convert_polygon(polygon: &PolygonType) -> Option<Polygon<GeoPoint2d>> {
    Some(Polygon::new(
        convert_contour(&polygon[0])?.into_closed()?,
        polygon[1..]
            .iter()
            .map(|p| convert_contour(p).and_then(|c| c.into_closed()))
            .collect::<Option<Vec<_>>>()?,
    ))
}

fn convert_multi_polygon(mp: &[PolygonType]) -> Option<MultiPolygon<GeoPoint2d>> {
    Some(MultiPolygon::from(
        mp.iter().map(convert_polygon).collect::<Option<Vec<_>>>()?,
    ))
}
