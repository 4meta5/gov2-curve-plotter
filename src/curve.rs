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
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Clone, Copy)]
pub struct TimeLength {
    pub unit: Time,
    pub length: u32,
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

/// Use this to generate the graph
pub struct CurvePoints {
    pub id: u16,
    pub name: String,
    pub unit: Time,
    // Continuous threshold coordinates for every unit increment of decision period length
    pub coordinates: Vec<Point>,
    // Coordinate with lowest threshold
    pub coordinate_threshold_min: Point,
    // Coordinate with highest threshold
    pub coordinate_threshold_max: Point,
    // Times for requested explicit thresholds
    pub thresholds: Vec<Point>,
}

impl CurvePoints {
    pub fn new(id: u16, name: String, time: TimeLength, curve: &Curve) -> Self {
        // to dedup
        let mut all_x_values = Vec::new();
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
                all_x_values.push(x);
                Point { x, y }
            })
            .collect();
        // TODO: generate in better way, not hardcoded?
        let mut all_thresholds: Vec<Perbill> = (0..100).map(|x| Perbill::from_percent(x)).collect();
        let mut rational_thresholds = vec![
            Perbill::from_rational(999u32, 1_000), //99.9%
            Perbill::from_rational(1u32, 1_000),   //0.1%
            Perbill::from_rational(1u32, 10_000),  //0.01%
        ];
        all_thresholds.append(&mut rational_thresholds);
        let thresholds: Vec<Point> = all_thresholds
            .into_iter()
            .filter_map(|y| {
                if y > y_min.y
                    && y < y_max.y
                    && !all_x_values.contains(&(curve.delay(y) * time.length))
                {
                    all_x_values.push(curve.delay(y) * time.length);
                    Some(Point {
                        x: curve.delay(y) * time.length,
                        y,
                    })
                } else {
                    None
                }
            })
            .collect();
        CurvePoints {
            id,
            name,
            unit: time.unit,
            coordinates,
            coordinate_threshold_min: y_min,
            coordinate_threshold_max: y_max,
            thresholds,
        }
    }
    /// Returns all points (for writing to csv)
    fn points(&self) -> Vec<Point> {
        let points: Vec<Point> = self
            .coordinates
            .clone()
            .into_iter()
            .chain(vec![self.coordinate_threshold_min, self.coordinate_threshold_max].into_iter())
            .chain(self.thresholds.clone().into_iter())
            .collect();
        assert_eq!(
            self.coordinates.len() + 2 + self.thresholds.len(),
            points.len(),
            "chain created vec of inconsistent len"
        );
        points
    }
    /// Return rounded points for plotting purposes
    fn rounded_points(&self) -> Vec<(u32, i32)> {
        self.coordinates
            .iter()
            .map(|Point { x, y }| (*x, perbill_to_percent_coordinate(*y)))
            .collect()
    }
    /// Implied decision period time length
    fn time_length(&self) -> u32 {
        self.coordinates.len() as u32
    }
    pub fn write_to_csv(&self, curve_ty: CurveType) {
        let path = match curve_ty {
            CurveType::Approval => format!("points/{} Approval.csv", self.name),
            CurveType::Support => format!("points/{} Support.csv", self.name),
        };
        let mut file = File::create(path).unwrap();
        for Point { x, y } in self.points() {
            file.write_all(format!("{}, {:?}\n", x, y).as_bytes())
                .unwrap();
        }
    }
    pub fn plot(&self, curve_ty: CurveType) {
        let (plot_png, chart_title, y_axis_label) = match curve_ty {
            CurveType::Approval => (
                format!("plots/{} Approval.png", self.name),
                format!("{} Approval, TrackID #{}", self.name, self.id),
                "% of Votes in Favor / All Votes in This Referendum",
            ),
            CurveType::Support => (
                format!("plots/{} Support.png", self.name),
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
            .build_cartesian_2d(0..self.time_length() + 20, chart_y_min..chart_y_max)
            .unwrap();
        let x_axis_label = match self.unit {
            Time::Day => format!("Days into {}-Day Decision Period", self.time_length()),
            Time::Hour => format!("Hours into {}-Day Decision Period", self.time_length() / 24),
            Time::Minute => format!(
                "Minutes into {}-Day Decision Period",
                (self.time_length() / 24 * 60)
            ),
            Time::Second => format!(
                "Seconds into {}-Day Decision Period",
                (self.time_length() / (24 * 60 * 60))
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
            assert!(t == curve.unit, "All curves must have consistent x units");
        } else {
            time = Some(curve.unit);
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
    let (plot_png, plot_title, y_axis_label) = match ty {
        CurveType::Approval => (
            "plots/Approvals.png",
            "Approval Requirements",
            "% of Votes in Favor / All Votes in This Referendum",
        ),
        CurveType::Support => (
            "plots/Supports.png",
            "Support Requirements",
            "% of Votes in This Referendum / Total Possible Turnout",
        ),
    };
    let grid = BitMapBackend::new(&plot_png, (1024, 768)).into_drawing_area();
    grid.fill(&WHITE).unwrap();
    let time = decision_period_time(&curves);
    let mut plot = ChartBuilder::on(&grid)
        .caption(&plot_title, ("sans-serif", 45))
        .margin(5)
        .set_left_and_bottom_label_area_size(60)
        .build_cartesian_2d(0..time.length, 0..100)
        .unwrap();
    let x_axis_label = match time.unit {
        Time::Day => format!("Days into {}-Day Decision Period", time.length),
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
