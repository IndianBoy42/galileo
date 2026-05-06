use approx::AbsDiffEq;
use serde::{Deserialize, Serialize};

use crate::cartesian::Point2;
use crate::geo::traits::point::{GeoPoint, NewGeoPoint};
use crate::geo::Datum;
use crate::geometry_type::{GeoSpace2d, GeometryType, PointGeometryType};

/// 2d point on the surface of a celestial body.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct GeoPoint2d {
    lat: f64,
    lon: f64,
}

impl GeoPoint for GeoPoint2d {
    /// Numeric type used to represent coordinates.
    type Num = f64;

    /// Latitude in degrees.
    fn lat(&self) -> f64 {
        self.lat
    }

    /// Longitude in degrees.
    fn lon(&self) -> f64 {
        self.lon
    }
}

impl NewGeoPoint<f64> for GeoPoint2d {
    fn latlon(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

impl AbsDiffEq for GeoPoint2d {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        f64::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.lat.abs_diff_eq(&other.lat, epsilon) && self.lon.abs_diff_eq(&other.lon, epsilon)
    }
}

const EARTH_RADIUS: f64 = 6371008.8; // WGS84 mean radius in meters

impl GeoPoint2d {
    /// Creates a new from point from another.
    pub fn from(other: &impl GeoPoint<Num = f64>) -> Self {
        Self {
            lat: other.lat(),
            lon: other.lon(),
        }
    }

    /// Add a offset defined in meters
    pub fn offset(self, dx: f64, dy: f64) -> Self {
        let delta_lat = (dy / EARTH_RADIUS).to_degrees();
        let delta_lon = (dx / (EARTH_RADIUS * self.lat_rad().cos())).to_degrees();

        Self::latlon(self.lat() + delta_lat, self.lon() + delta_lon)
    }
    /// Get the cartesian offset from `self` to `other` in meters
    pub fn get_offset_to(self, other: &GeoPoint2d) -> Point2 {
        Point2::new(
            (other.lon() - self.lon()).to_radians() * EARTH_RADIUS * self.lat_rad().cos(),
            (other.lat() - self.lat()).to_radians() * EARTH_RADIUS,
        )
    }
    // TODO: get_offset_accurate
    /// Add an offset defined in meters, more accurate for a larger distance using datum parameters
    pub fn offset_accurate(self, dx: f64, dy: f64, datum: &Datum) -> Self {
        let distance = (dx.powi(2) + dy.powi(2)).sqrt();
        let azimuth = dy.atan2(dx).to_degrees();

        let a = datum.semimajor();
        let b = datum.semiminor();
        let f = 1.0 - (b / a); // flattening

        let α1 = azimuth.to_radians();
        let sin_α1 = α1.sin();
        let cos_α1 = α1.cos();

        let tan_u1 = (1.0 - f) * self.lat_rad().tan();
        let cos_u1 = 1.0 / (1.0 + tan_u1.powi(2)).sqrt();
        let sin_u1 = tan_u1 * cos_u1;
        let σ1 = sin_u1.atan2(cos_u1);
        let sin_α = cos_u1 * sin_α1;
        let cos_sq_α = 1.0 - sin_α.powi(2);
        let u_sq = cos_sq_α * (a.powi(2) - b.powi(2)) / b.powi(2);

        let mut σ = distance
            / (b * (1.0
                + u_sq / 16384.0 * (4096.0 + u_sq * (-768.0 + u_sq * (320.0 - 175.0 * u_sq)))));
        let mut sin_σ = σ.sin();
        let mut cos_σ = σ.cos();

        for _ in 0..10 {
            let cos_2σ_m = cos_σ * (-1.0 + 2.0 * σ1.cos().powi(2));
            let δσ = (u_sq / 1024.0)
                * (256.0 + u_sq * (-128.0 + u_sq * (74.0 - 47.0 * u_sq)))
                * (cos_2σ_m * (-3.0 + 4.0 * sin_σ.powi(2)) * (-3.0 + 4.0 * cos_2σ_m.powi(2)) / 6.0
                    + cos_σ * (-1.0 + 2.0 * cos_2σ_m.powi(2)) / 4.0);

            let σ_new = σ + δσ;
            if (σ_new - σ).abs() < 1e-12 {
                break;
            }
            σ = σ_new;
            sin_σ = σ.sin();
            cos_σ = σ.cos();
        }

        let cos_2σ_m = cos_σ * (-1.0 + 2.0 * σ1.cos().powi(2));
        let tmp = sin_u1 * sin_σ - cos_u1 * cos_σ * cos_α1;
        let lat2 = (sin_u1 * cos_σ + cos_u1 * sin_σ * cos_α1)
            .atan2((1.0 - f) * (sin_α.powi(2) + tmp.powi(2)).sqrt());
        let λ = (sin_σ * sin_α1).atan2(cos_u1 * cos_σ - sin_u1 * sin_σ * cos_α1);
        let c = (f / 16.0) * cos_sq_α * (4.0 + f * (4.0 - 3.0 * cos_sq_α));
        let l = λ
            - (1.0 - c)
                * f
                * sin_α
                * (σ + c * sin_σ * (cos_2σ_m + c * cos_σ * (-1.0 + 2.0 * cos_2σ_m.powi(2))));

        let new_lat = lat2.to_degrees();
        let new_lon = self.lon() + l.to_degrees();

        Self::latlon(new_lat, new_lon)
    }

    /// Move point by distance (meters) in direction specified by heading (degrees clockwise from North)
    pub fn offset_polar(self, distance: f64, heading: f64) -> Self {
        // TODO: direct method
        let heading_rad = heading.to_radians();
        let dx = distance * heading_rad.sin();
        let dy = distance * heading_rad.cos();
        self.offset(dx, dy)
    }

    pub fn offset_polar_accurate(self, distance: f64, heading: f64, datum: &Datum) -> Self {
        // TODO: direct method
        let heading_rad = heading.to_radians();
        let dx = distance * heading_rad.sin();
        let dy = distance * heading_rad.cos();
        self.offset_accurate(dx, dy, datum)
    }
}

impl GeometryType for GeoPoint2d {
    type Type = PointGeometryType;
    type Space = GeoSpace2d;
}

/// Creates a new GeoPoint2d from latitude and longitude values (in degrees).
///
/// ```
/// use galileo_types::geo::GeoPoint;
/// use galileo_types::latlon;
///
/// let point = latlon!(38.0, 52.0);
/// assert_eq!(point.lat(), 38.0);
/// ```
#[macro_export]
macro_rules! latlon {
    ($lat:expr, $lon:expr) => {
        <$crate::geo::impls::GeoPoint2d as $crate::geo::NewGeoPoint<f64>>::latlon($lat, $lon)
    };
}
