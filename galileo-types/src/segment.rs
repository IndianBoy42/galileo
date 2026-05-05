use num_traits::{One, Zero};

use crate::cartesian::{CartesianPoint2d, Orientation};

/// A strait line segment between two points.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Segment<Point>(pub Point, pub Point);

impl<P: CartesianPoint2d> Segment<P> {
    /// Shortest euclidean distance (squared) between a point and the segment:
    ///
    /// * if the normal from the point to the segment ends inside the segment, the returned value is the squared length
    ///   of the normal
    /// * if the normal from the point to the segment ends outside of the segment, the returned value is the smaller one
    ///   of the distances between the point and the segment's endpoints
    pub fn distance_to_point_sq<Point: CartesianPoint2d<Num = P::Num>>(
        &self,
        point: &Point,
    ) -> P::Num {
        if self.0.equal(&self.1) {
            return self.0.distance_sq(point);
        }

        let ds = self.1.sub(&self.0);
        let dp = point.sub(&self.0);
        let ds_len = ds.magnitude_sq();

        let r = dp.magnitude_sq() / ds_len;
        if r <= P::Num::zero() {
            self.0.distance_sq(point)
        } else if r >= P::Num::one() {
            self.1.distance_sq(point)
        } else {
            let s = (dp.dy() * ds.dx() - dp.dx() * ds.dy()) / ds_len;
            (s * s) * ds_len
        }
    }

    /// Returns true, if the segment has at least one common point with the `other` segment.
    pub fn intersects<Point: CartesianPoint2d<Num = P::Num>>(
        &self,
        other: &Segment<Point>,
    ) -> bool {
        fn on_segment<Num: num_traits::Num + PartialOrd>(
            p: &impl CartesianPoint2d<Num = Num>,
            q: &impl CartesianPoint2d<Num = Num>,
            r: &impl CartesianPoint2d<Num = Num>,
        ) -> bool {
            let x_max = if p.x() >= r.x() { p.x() } else { r.x() };
            let x_min = if p.x() <= r.x() { p.x() } else { r.x() };
            let y_max = if p.y() >= r.y() { p.x() } else { r.x() };
            let y_min = if p.y() <= r.y() { p.x() } else { r.x() };

            q.x() <= x_max && q.x() >= x_min && q.y() <= y_max && q.y() >= y_min
        }

        let ori1 = Orientation::triplet(&self.0, &other.0, &self.1);
        let ori2 = Orientation::triplet(&self.0, &other.1, &self.1);
        let ori3 = Orientation::triplet(&other.0, &self.0, &other.1);
        let ori4 = Orientation::triplet(&other.0, &self.1, &other.1);

        if ori1 != ori2 && ori3 != ori4 {
            return true;
        }

        if ori1 == Orientation::Collinear && on_segment(&self.0, &other.0, &self.1) {
            return true;
        }
        if ori2 == Orientation::Collinear && on_segment(&self.0, &other.1, &self.1) {
            return true;
        }
        if ori3 == Orientation::Collinear && on_segment(&other.0, &self.0, &other.1) {
            return true;
        }
        if ori4 == Orientation::Collinear && on_segment(&other.0, &self.1, &other.1) {
            return true;
        }

        false
    }
}
