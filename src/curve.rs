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

pub enum TimeIncrement {
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Clone)]
pub struct Point<Y> {
    pub x: u32,
    pub y: Y,
}

impl<T> From<(u32, T)> for Point<T> {
    fn from(point: (u32, T)) -> Point<T> {
        Point {
            x: point.0,
            y: point.1,
        }
    }
}

pub struct Points<Y> {
    pub coordinates: Vec<Point<Y>>,
    pub x_unit: TimeIncrement,
}

pub struct CurveInfo {
    pub track_id: u16,
    pub name: String,
    pub points: Points<i32>,
    // y_min, y_max
    pub y_range: Points<Perbill>,
    // integer thresholds
    pub thresholds: Points<Perbill>,
}

impl CurveInfo {
    pub fn points(&self) -> Vec<Point<i32>> {
        self.points.coordinates.clone()
    }
    pub fn raw_points(&self) -> Vec<(u32, i32)> {
        self.points()
            .iter()
            .map(|point| (point.x, point.y))
            .collect()
    }
    /// Plots input curve, writes points to csv & returns relevant curve info
    pub fn new(name: String, id: u16, curve_ty: CurveType, curve: &Curve, hours: u32) -> Self {
        let (plot_png, points_csv, chart_title, y_axis_label) = match curve_ty {
            CurveType::Approval => (
                format!("plots/{} Approval.png", name),
                format!("points/{} Approval.csv", name),
                format!("{} Approval, TrackID #{}", name, id),
                "% of Votes in Favor / All Votes in This Referendum",
            ),
            CurveType::Support => (
                format!("plots/{} Support.png", name),
                format!("points/{} Support.csv", name),
                format!("{} Support, TrackID #{}", name, id),
                "% of Votes in This Referendum / Total Possible Turnout",
            ),
        };
        let grid = BitMapBackend::new(&plot_png, (600, 400)).into_drawing_area();
        grid.fill(&WHITE).unwrap();
        // TODO: improve this, can check around curve points right?
        let (mut y_min_coordinate, mut y_max_coordinate) =
            ((0u32, Perbill::one()), (0u32, Perbill::zero()));
        let mut curve_points: Vec<(u32, Perbill)> = (0..=hours)
            .map(|x| {
                let y = threshold(curve, Perbill::from_rational(x, hours));
                if y > y_max_coordinate.1 {
                    y_max_coordinate = (x, y);
                }
                if y < y_min_coordinate.1 {
                    y_min_coordinate = (x, y);
                }
                (x, y)
            })
            .collect();
        // TODO: do not lose precision
        let (y_min, y_max) = (
            perbill_to_percent_coordinate(y_min_coordinate.1),
            perbill_to_percent_coordinate(y_max_coordinate.1),
        );
        let chart_y_min = if y_min > 10i32 { y_min - 10i32 } else { 0i32 };
        let chart_y_max = if y_max < 90i32 { y_max + 10i32 } else { 100i32 };
        let mut chart = ChartBuilder::on(&grid)
            .caption(&chart_title, ("sans-serif", 30))
            .margin(5)
            .set_left_and_bottom_label_area_size(40)
            .build_cartesian_2d(0..hours + 20, chart_y_min..chart_y_max)
            .unwrap();
        let x_axis_label = format!("Hours into {}-Day Decision Period", hours / 24);
        chart
            .configure_mesh()
            .y_desc(y_axis_label)
            .x_desc(x_axis_label)
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .unwrap();
        let points = curve_points
            .clone()
            .into_iter()
            .map(move |(x, y)| (x, perbill_to_percent_coordinate(y)));
        chart
            .draw_series(LineSeries::new(points.clone(), &RED))
            .unwrap();
        // TODO: generate in better way, not hardcoded?
        let mut thresholds: Vec<Perbill> = (0..100).map(|x| Perbill::from_percent(x)).collect();
        let mut rational_thresholds = vec![
            Perbill::from_rational(999u32, 1_000), //99.9%
            Perbill::from_rational(1u32, 1_000),   //0.1%
            Perbill::from_rational(1u32, 10_000),  //0.01%
        ];
        thresholds.append(&mut rational_thresholds);
        let mut dedup_threshold_points = Vec::new();
        let mut threshold_points: Vec<(u32, Perbill)> = thresholds
            .into_iter()
            .filter_map(|y| {
                if y > y_min_coordinate.1
                    && y < y_max_coordinate.1
                    && !dedup_threshold_points.contains(&(curve.delay(y) * hours))
                {
                    dedup_threshold_points.push(curve.delay(y) * hours);
                    Some((curve.delay(y) * hours, y))
                } else {
                    None
                }
            })
            .collect();
        curve_points.append(&mut threshold_points);
        write_curve_points_csv(points_csv, curve_points);
        Self {
            track_id: id,
            name,
            points: Points {
                coordinates: points.map(|x| x.into()).collect(),
                x_unit: TimeIncrement::Hour,
            },
            y_range: Points {
                coordinates: vec![y_min_coordinate, y_max_coordinate]
                    .into_iter()
                    .map(|x| x.into())
                    .collect(),
                x_unit: TimeIncrement::Hour,
            },
            thresholds: Points {
                coordinates: threshold_points.into_iter().map(|x| x.into()).collect(),
                x_unit: TimeIncrement::Hour,
            },
        }
    }
}

pub type Curves = Vec<CurveInfo>;

/// Assumes all curves have same decision period
fn decision_period(curves: &Curves) -> u32 {
    let hours = curves[0].points().len();
    for curve in curves {
        // plus thresholds
        if curve.points().len() != hours {
            panic!("Decision Period not constant for all curves");
        }
    }
    hours.try_into().unwrap()
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
    let hours = decision_period(&curves);
    let mut plot = ChartBuilder::on(&grid)
        .caption(&plot_title, ("sans-serif", 45))
        .margin(5)
        .set_left_and_bottom_label_area_size(60)
        .build_cartesian_2d(0..hours, 0..100)
        .unwrap();
    plot.configure_mesh()
        .y_desc(y_axis_label)
        .x_desc(format!("Hours into {}-Day Decision Period", hours / 24))
        .axis_desc_style(("sans-serif", 30))
        .draw()
        .unwrap();
    for (i, curve) in curves.iter().enumerate() {
        let color = Palette99::pick(i).mix(0.9);
        plot.draw_series(LineSeries::new(
            curve.raw_points().clone(),
            color.stroke_width(2),
        ))
        .unwrap()
        .label(format!("{}, ID # {}", curve.name, curve.track_id))
        .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
    }
    plot.configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

/// Write curve points to file
fn write_curve_points_csv(file: String, points: Vec<(u32, Perbill)>) {
    let mut file = File::create(file).unwrap();
    for (x, y) in points {
        file.write_all(format!("{}, {:?}\n", x, y).as_bytes())
            .unwrap();
    }
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
