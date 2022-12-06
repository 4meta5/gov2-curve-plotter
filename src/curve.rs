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

pub struct CurveInfo {
    pub track_id: u16,
    pub name: String,
    pub points: Vec<(u32, i32)>,
}

pub type Curves = Vec<CurveInfo>;

/// Assumes all curves have same decision period
fn decision_period(curves: &Curves) -> u32 {
    let hours = curves[0].points.len();
    for curve in curves {
        // plus thresholds
        if curve.points.len() != hours {
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
        plot.draw_series(LineSeries::new(curve.points.clone(), color.stroke_width(2)))
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

pub(crate) fn plot_curve(
    name: String,
    id: u16,
    curve_ty: CurveType,
    curve: &Curve,
    hours: u32,
) -> Vec<(u32, i32)> {
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
    let (mut points_min, mut points_max) = ((0u32, Perbill::one()), (0u32, Perbill::zero()));
    let curve_points: Vec<(u32, Perbill)> = (0..=hours)
        .map(|x| {
            let y = threshold(curve, Perbill::from_rational(x, hours));
            if y > points_max.1 {
                points_max = (x, y);
            }
            if y < points_min.1 {
                points_min = (x, y);
            }
            (x, y)
        })
        .collect();
    let (mut y_min, mut y_max) = (
        perbill_to_percent_coordinate(points_min.1),
        perbill_to_percent_coordinate(points_max.1),
    );
    if y_min > 5i32 {
        y_min -= 2i32;
    } else {
        y_min = 0i32;
    };
    if y_max < 95i32 {
        y_max += 2i32;
    } else {
        y_max = 100i32;
    }
    let mut chart = ChartBuilder::on(&grid)
        .caption(&chart_title, ("sans-serif", 30))
        .margin(5)
        .set_left_and_bottom_label_area_size(40)
        .build_cartesian_2d(0..hours, y_min..y_max)
        .unwrap();
    let x_axis_label = format!("Hours into {}-Day Decision Period", hours / 24);
    chart
        .configure_mesh()
        .y_desc(y_axis_label)
        .x_desc(x_axis_label)
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .unwrap();
    write_curve_points_csv(points_csv, curve_points.clone());
    let points = curve_points
        .into_iter()
        .map(move |(x, y)| (x, perbill_to_percent_coordinate(y)));
    chart
        .draw_series(LineSeries::new(points.clone(), &RED))
        .unwrap();
    // TODO: minute, second accuracy
    // TODO: functions on top which take input starting_time and output threshold_times
    // get min and max for points and then use that to educate thresholds filter
    // make these depend on the range, and also make the chart depend on range
    let thresholds: Vec<Perbill> = vec![99, 95, 90, 75, 60, 50, 40, 30, 25, 20, 10, 5, 1]
        .into_iter()
        .map(|x| Perbill::from_percent(x))
        .collect();
    // TODO: append points_min, points_max to threshold_points
    let mut dedup_threshold_points = Vec::new();
    let threshold_points: Vec<(u32, Perbill)> = thresholds
        .into_iter()
        .filter_map(|y| {
            // also need to filter out if the threshold point already exists
            if y > points_min.1
                && y < points_max.1
                && !dedup_threshold_points.contains(&(curve.delay(y) * hours))
            {
                dedup_threshold_points.push(curve.delay(y) * hours);
                Some((curve.delay(y) * hours, y))
            } else {
                None
            }
        })
        .collect();
    // include min/max in threshold points
    // then make threshold points more legitimate
    chart
        .draw_series(PointSeries::of_element(
            threshold_points
                .into_iter()
                .map(move |(x, y)| (x, perbill_to_percent_coordinate(y))),
            5,
            ShapeStyle::from(&BLUE).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(format!("{:?}", coord), (0, 15), ("sans-serif", 15))
            },
        ))
        .unwrap();
    // TODO: include threshold points in return value
    points.collect()
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
