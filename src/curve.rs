// TODO:
// 1. include key_threshold_times: Vec<Perbill>
// 2. write to csv once for all approval, support curves
// 3. each graph should have Y range based on the min/max instead of 0 to 100
// 4. only include the specified curves
// 5. plot individual curve, use this function for the other functions too if possible
use pallet_referenda::Curve;
use plotters::prelude::*;
use sp_arithmetic::{Rounding::*, SignedRounding::*};
use sp_runtime::{FixedI64, Perbill};
use std::fs::File;
use std::io::Write;

#[derive(Clone, Copy)]
pub enum CurveType {
    /// Approval is defined as the share of approval vote-weight (i.e. after adjustment for
    /// conviction) against the total number of vote-weight (for both approval and rejection).
    Approval,
    /// Support is the total number of votes in approval (i.e. ignoring any adjustment for
    /// conviction) compared to the total possible amount of votes that could be made in the system.
    Support,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Time {
    Hour,
    Minute,
    #[allow(dead_code)]
    Second,
}

impl core::fmt::Display for Time {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            Time::Hour => f.write_str("Hour"),
            Time::Minute => f.write_str("Minute"),
            Time::Second => f.write_str("Second"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TimeLength {
    pub unit: Time,
    pub length: u32,
}

impl TimeLength {
    fn to_seconds(&self) -> TimeLength {
        match self.unit {
            Time::Hour => TimeLength {
                unit: Time::Second,
                length: self.length * 60 * 60,
            },
            Time::Minute => TimeLength {
                unit: Time::Minute,
                length: self.length * 60,
            },
            Time::Second => *self,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: u32,
    pub y: Perbill,
}

impl From<(u32, Perbill)> for Point {
    fn from(point: (u32, Perbill)) -> Point {
        Point {
            x: point.0,
            y: point.1,
        }
    }
}

#[derive(Clone)]
pub struct Points {
    pub time_length: TimeLength,
    pub points: Vec<Point>,
}

/// Generated from a Curve input, see the `new` function for details
pub struct CurvePoints {
    /// Approval or Support curve
    pub curve_ty: CurveType,
    /// TrackId
    pub id: u16,
    /// Track name
    pub name: String,
    /// Per time increment thresholds
    /// By default time increment is per hour, but can do per minute, second
    pub coordinates: Points,
    pub coordinate_threshold_min: Point,
    pub coordinate_threshold_max: Point,
    /// Hardcoded human-readable threshold points i.e. 99.9%, 0.1%, 50%
    /// By default per-second thresholds for hardcoded thresholds
    pub thresholds: Points,
}

impl CurvePoints {
    pub fn new(
        curve_ty: CurveType,
        id: u16,
        name: String,
        time: TimeLength,
        curve: &Curve,
    ) -> Self {
        let (mut y_min, mut y_max) = (
            Point {
                x: 0u32,
                y: Perbill::one(),
            },
            Point {
                x: 0u32,
                y: Perbill::zero(),
            },
        );
        let coordinates: Vec<Point> = (0..=time.length)
            .map(|x| {
                let y = threshold(curve, Perbill::from_rational(x, time.length));
                if y > y_max.y {
                    y_max = Point { x, y };
                }
                if y < y_min.y {
                    y_min = Point { x, y };
                }
                Point { x, y }
            })
            .collect();
        // TODO: generate in better way, not hardcoded?
        let mut thresholds: Vec<Perbill> = (0..100).map(|x| Perbill::from_percent(x)).collect();
        let mut rational_thresholds = vec![
            Perbill::from_rational(999u32, 1_000), //99.9%
            Perbill::from_rational(1u32, 1_000),   //0.1%
            Perbill::from_rational(1u32, 10_000),  //0.01%
        ];
        thresholds.append(&mut rational_thresholds);
        // always use second for the thresholds
        // TODO: save this somewhere for thresholds or write it to the file
        let seconds_length_is_thresholds_length = time.to_seconds();
        let thresholds: Vec<Point> = thresholds
            .into_iter()
            .filter_map(|y| {
                if y > y_min.y && y < y_max.y {
                    // thresholds are always in points
                    Some(Point {
                        x: curve.delay(y) * seconds_length_is_thresholds_length.length,
                        y,
                    })
                } else {
                    None
                }
            })
            .collect();
        CurvePoints {
            curve_ty,
            id,
            name,
            coordinates: Points {
                time_length: time,
                points: coordinates,
            },
            coordinate_threshold_min: y_min,
            coordinate_threshold_max: y_max,
            thresholds: Points {
                time_length: seconds_length_is_thresholds_length,
                points: thresholds,
            },
        }
    }
    /// Returns all points (for writing to csv)
    fn points(&self) -> Vec<Point> {
        self.coordinates.points.clone()
    }
    /// Return rounded points for plotting purposes
    fn rounded_points(&self) -> Vec<(u32, i32)> {
        self.points()
            .iter()
            .map(|Point { x, y }| (*x, perbill_to_percent_coordinate(*y)))
            .collect()
    }
    // Writes coordinates to CSV
    // file name uniqueness schema: {name} Approval/Support {Time::Hour}
    pub fn write_to_csv(&self) {
        let (coordinate_path, threshold_path) = match self.curve_ty {
            CurveType::Approval => (
                format!(
                    "points/{} Approval {}.csv",
                    self.name, self.coordinates.time_length.unit
                ),
                format!(
                    "points/{} Approval Thresholds{}.csv",
                    self.name, self.thresholds.time_length.unit
                ),
            ),
            CurveType::Support => (
                format!(
                    "points/{} Support {}.csv",
                    self.name, self.coordinates.time_length.unit
                ),
                format!(
                    "points/{} Support Thresholds{}.csv",
                    self.name, self.thresholds.time_length.unit
                ),
            ),
        };
        let (mut coordinate_file, mut threshold_file) = (
            File::create(coordinate_path).unwrap(),
            File::create(threshold_path).unwrap(),
        );
        for Point { x, y } in &self.coordinates.points {
            coordinate_file
                .write_all(format!("{}, {:?}\n", x, y).as_bytes())
                .unwrap();
        }
        threshold_file
            .write_all(
                format!(
                    "TOTAL SECONDS (X): {}\n",
                    self.thresholds.time_length.length
                )
                .as_bytes(),
            )
            .unwrap();
        for Point { x, y } in &self.thresholds.points {
            threshold_file
                .write_all(format!("{}, {:?}\n", x, y).as_bytes())
                .unwrap();
        }
    }
    pub fn plot(&self) {
        let (plot_png, chart_title, y_axis_label) = match self.curve_ty {
            CurveType::Approval => (
                format!(
                    "plots/{} Approval {}.png",
                    self.name, self.coordinates.time_length.unit
                ),
                format!("{} Approval, TrackID #{}", self.name, self.id),
                "% of Votes in Favor / All Votes in This Referendum",
            ),
            CurveType::Support => (
                format!(
                    "plots/{} Support {}.png",
                    self.name, self.coordinates.time_length.unit
                ),
                format!("{} Support, TrackID #{}", self.name, self.id),
                "% of Votes in This Referendum / Total Possible Turnout",
            ),
        };
        let grid = BitMapBackend::new(&plot_png, (600, 400)).into_drawing_area();
        grid.fill(&WHITE).unwrap();
        // TODO: do not lose precision
        let (y_min, y_max) = (
            perbill_to_percent_coordinate(self.coordinate_threshold_min.y),
            perbill_to_percent_coordinate(self.coordinate_threshold_max.y),
        );
        let chart_y_min = if y_min > 10i32 { y_min - 10i32 } else { 0i32 };
        let chart_y_max = if y_max < 90i32 { y_max + 10i32 } else { 100i32 };
        let mut chart = ChartBuilder::on(&grid)
            .caption(&chart_title, ("sans-serif", 30))
            .margin(5)
            .set_left_and_bottom_label_area_size(40)
            .build_cartesian_2d(
                0..self.coordinates.time_length.length + 20,
                chart_y_min..chart_y_max,
            )
            .unwrap();
        let x_axis_label = match self.coordinates.time_length.unit {
            Time::Hour => format!(
                "Hours into {}-Day Decision Period",
                self.coordinates.time_length.length / 24
            ),
            Time::Minute => format!(
                "Minutes into {}-Day Decision Period",
                (self.coordinates.time_length.length / 24 * 60)
            ),
            Time::Second => format!(
                "Seconds into {}-Day Decision Period",
                (self.coordinates.time_length.length / (24 * 60 * 60))
            ),
        };
        chart
            .configure_mesh()
            .y_desc(y_axis_label)
            .x_desc(x_axis_label)
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .unwrap();
        chart
            .draw_series(LineSeries::new(self.rounded_points(), &RED))
            .unwrap();
    }
}

pub type Curves = Vec<CurvePoints>;

/// Assumes all curves have same decision period
fn decision_period_time(curves: &Curves) -> TimeLength {
    let length = curves[0].points().len();
    let mut time: Option<Time> = None;
    for curve in curves {
        if curve.points().len() != length {
            panic!("Decision Period not constant for all curves");
        }
        if let Some(t) = time {
            assert!(
                t == curve.coordinates.time_length.unit,
                "All curves must have consistent x units"
            );
        } else {
            time = Some(curve.coordinates.time_length.unit);
        }
    }
    TimeLength {
        unit: time.unwrap(),
        length: length.try_into().unwrap(),
    }
}

pub(crate) fn plot_approval_curves(curves: Curves) {
    plot_curves(CurveType::Approval, curves)
}

pub(crate) fn plot_support_curves(curves: Curves) {
    plot_curves(CurveType::Support, curves)
}

fn plot_curves(ty: CurveType, curves: Curves) {
    let time = decision_period_time(&curves);
    let (plot_png, plot_title, y_axis_label) = match ty {
        CurveType::Approval => (
            format!("plots/Approvals {}.png", time.unit),
            "Approval Requirements",
            "% of Votes in Favor / All Votes in This Referendum",
        ),
        CurveType::Support => (
            format!("plots/Supports {}.png", time.unit),
            "Support Requirements",
            "% of Votes in This Referendum / Total Possible Turnout",
        ),
    };
    let grid = BitMapBackend::new(&plot_png, (1024, 768)).into_drawing_area();
    grid.fill(&WHITE).unwrap();
    let mut plot = ChartBuilder::on(&grid)
        .caption(&plot_title, ("sans-serif", 45))
        .margin(5)
        .set_left_and_bottom_label_area_size(60)
        .build_cartesian_2d(0..time.length, 0..100)
        .unwrap();
    let x_axis_label = match time.unit {
        Time::Hour => format!("Hours into {}-Day Decision Period", time.length / 24),
        Time::Minute => format!(
            "Minutes into {}-Day Decision Period",
            (time.length / 24 * 60)
        ),
        Time::Second => format!(
            "Seconds into {}-Day Decision Period",
            (time.length / (24 * 60 * 60))
        ),
    };
    plot.configure_mesh()
        .y_desc(y_axis_label)
        .x_desc(x_axis_label)
        .axis_desc_style(("sans-serif", 30))
        .draw()
        .unwrap();
    for (i, curve) in curves.iter().enumerate() {
        let color = Palette99::pick(i).mix(0.9);
        plot.draw_series(LineSeries::new(
            curve.rounded_points().clone(),
            color.stroke_width(2),
        ))
        .unwrap()
        .label(format!("{}, ID # {}", curve.name, curve.id))
        .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
    }
    plot.configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

/// Input Perbill, output i32 between 0 and 100
fn perbill_to_percent_coordinate(input: Perbill) -> i32 {
    (input.deconstruct() / (Perbill::one().deconstruct() / 100))
        .try_into()
        .unwrap()
}

#[test]
fn perbill_to_i32_percent_conversion() {
    for i in 0..100 {
        let j: i32 = i.into();
        assert_eq!(perbill_to_percent_coordinate(Perbill::from_percent(i)), j);
    }
}

/// Determine the `y` value for the given `x` value.
fn threshold(curve: &Curve, x: Perbill) -> Perbill {
    match curve {
        Curve::LinearDecreasing {
            length,
            floor,
            ceil,
        } => *ceil - (x.min(*length).saturating_div(*length, Down) * (*ceil - *floor)),
        Curve::SteppedDecreasing {
            begin,
            end,
            step,
            period,
        } => (*begin - (step.int_mul(x.int_div(*period))).min(*begin)).max(*end),
        Curve::Reciprocal {
            factor,
            x_offset,
            y_offset,
        } => factor
            .checked_rounding_div(FixedI64::from(x) + *x_offset, Low)
            .map(|yp| (yp + *y_offset).into_clamped_perthing())
            .unwrap_or_else(Perbill::one),
    }
}
