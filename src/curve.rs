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
        if curve.points.len() != hours {
            panic!("Decision Period not constant for all curves");
        }
    }
    hours.try_into().unwrap()
}

pub(crate) fn plot_approval_curves(curves: Curves) {
    plot_curves_comparison(CurveType::Approval, curves)
}

pub(crate) fn plot_support_curves(curves: Curves) {
    plot_curves_comparison(CurveType::Support, curves)
}

fn plot_curves_comparison(ty: CurveType, curves: Curves) {
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

// TODO
fn plot_curves(ty: CurveType, curves: Curves, write_points_to_csv: bool) {
    todo!()
}

pub(crate) fn plot_curve(
    name: String,
    id: u16,
    curve_ty: CurveType,
    curve: &Curve,
    hours: u32,
    thresholds: Vec<Perbill>,
    write_points_to_csv: bool,
) -> Vec<(u32, i32)> {
    let (plot_png, chart_title) = match curve_ty {
        CurveType::Approval => (
            format!("plots/{} Approval.png", name),
            format!("{} Approval, TrackID #{}", name, id),
        ),
        CurveType::Support => (
            format!("plots/{} Support.png", name),
            format!("{} Support, TrackID #{}", name, id),
        ),
    };
    let grid = BitMapBackend::new(&plot_png, (600, 400)).into_drawing_area();
    grid.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&grid)
        .caption(&chart_title, ("sans-serif", 30))
        .margin(5)
        .set_left_and_bottom_label_area_size(40)
        // TODO: y label should be from y_min to y_max instead of 0-100
        .build_cartesian_2d(0..hours + 1, 0..100)
        .unwrap();
    let x_axis_label = format!("Hours into {}-Day Decision Period", hours / 24);
    match curve_ty {
        CurveType::Approval => chart
            .configure_mesh()
            .y_desc("% of Votes in Favor / All Votes in This Referendum")
            .x_desc(x_axis_label.clone())
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .unwrap(),
        CurveType::Support => chart
            .configure_mesh()
            .y_desc("% of Votes in This Referendum / Total Possible Turnout")
            .x_desc(x_axis_label.clone())
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .unwrap(),
    }
    let threshold_points: Vec<(u32, Perbill)> = thresholds
        .into_iter()
        .map(|y| (curve.delay(y) * hours, y))
        .collect();
    let curve_points =
        (0..=hours).map(move |x| (x, threshold(curve, Perbill::from_rational(x, hours))));
    // TODO: merge in order would be nicer csv output but easy to sort in excel
    let mut points_for_csv: Vec<(u32, Perbill)> = curve_points.clone().collect();
    points_for_csv.append(&mut threshold_points.clone());
    if write_points_to_csv {
        let points_csv = match curve_ty {
            CurveType::Approval => format!("points/{} Approval.csv", name),
            CurveType::Support => format!("points/{} Support.csv", name),
        };
        write_curve_points_csv(points_csv, points_for_csv);
    }
    let points = curve_points.map(move |(x, y)| (x, perbill_to_percent_coordinate(y)));
    chart
        .draw_series(LineSeries::new(points.clone(), &RED))
        .unwrap();
    chart
        .draw_series(PointSeries::of_element(
            threshold_points
                .into_iter()
                .map(move |(x, y)| (x, perbill_to_percent_coordinate(y))),
            5,
            ShapeStyle::from(&GREEN).filled(),
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
