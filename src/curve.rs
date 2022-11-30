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

pub(crate) fn plot_track_curves(
    name: String,
    id: u16,
    approval_curve: &Curve,
    support_curve: &Curve,
    days: u32,
) -> (Vec<(u32, i32)>, Vec<(u32, i32)>) {
    let (app_plot_png, sup_plot_png, app_points_csv, sup_points_csv) = (
        format!("plots/{} Approval.png", name),
        format!("plots/{} Support.png", name),
        format!("points/{} Approval.csv", name),
        format!("points/{} Support.csv", name),
    );
    let make_grid = |file_name| {
        let grid = BitMapBackend::new(file_name, (600, 400)).into_drawing_area();
        grid.fill(&WHITE).unwrap();
        grid
    };
    let app_grid = make_grid(&app_plot_png);
    let sup_grid = make_grid(&sup_plot_png);
    let hours = 24 * days;
    let make_plot = |grid, title| {
        ChartBuilder::on(grid)
            .caption(&title, ("sans-serif", 30))
            .margin(5)
            .set_left_and_bottom_label_area_size(40)
            .build_cartesian_2d(0..hours + 1, 0..100)
            .unwrap()
    };
    let mut app_plot = make_plot(&app_grid, format!("{} Approval, TrackID #{}", name, id));
    let mut sup_plot = make_plot(&sup_grid, format!("{} Support, TrackID #{}", name, id));
    let x_axis_label = format!("Hours into {}-Day Decision Period", days);
    app_plot
        .configure_mesh()
        .y_desc("% of Votes in Favor / All Votes in This Referendum")
        .x_desc(x_axis_label.clone())
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .unwrap();
    sup_plot
        .configure_mesh()
        .y_desc("% of Votes in This Referendum / Total Possible Turnout")
        .x_desc(x_axis_label)
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .unwrap();
    let curve_points =
        |crv, pts| (0..=pts).map(move |x| (x, threshold(crv, Perbill::from_rational(x, pts))));
    let approval_curve_points = curve_points(approval_curve, hours);
    let support_curve_points = curve_points(support_curve, hours);
    write_curve_points_csv(app_points_csv, approval_curve_points.clone().collect());
    write_curve_points_csv(sup_points_csv, support_curve_points.clone().collect());
    let rounded_approval_curve =
        approval_curve_points.map(move |(x, y)| (x, perbill_to_percent_coordinate(y)));
    let rounded_support_curve =
        support_curve_points.map(move |(x, y)| (x, perbill_to_percent_coordinate(y)));
    app_plot
        .draw_series(LineSeries::new(rounded_approval_curve.clone(), &RED))
        .unwrap();
    sup_plot
        .draw_series(LineSeries::new(rounded_support_curve.clone(), &RED))
        .unwrap();
    (
        rounded_approval_curve.collect(),
        rounded_support_curve.collect(),
    )
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
