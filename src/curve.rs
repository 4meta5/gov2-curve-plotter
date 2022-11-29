use pallet_referenda::Curve;
use plotters::prelude::*;
use sp_arithmetic::{Rounding::*, SignedRounding::*};
use sp_runtime::{FixedI64, Perbill};
use std::fs::File;
use std::io::Write;

/// Returns approval and support curve points for rendering all approval/support together
pub(crate) fn plot_track_curves(
    name: String,
    id: u16,
    // Approval is defined as the share of approval vote-weight (i.e. after adjustment for
    // conviction) against the total number of vote-weight (for both approval and rejection).
    approval_curve: &Curve,
    // Support is the total number of votes in approval (i.e. ignoring any adjustment for
    // conviction) compared to the total possible amount of votes that could be made in the system.
    support_curve: &Curve,
    days: u32,
) -> (Vec<(u32, i32)>, Vec<(u32, i32)>) {
    let (app_plot_png, sup_plot_png, app_points_csv, sup_points_csv) = (
        format!("plots/{} Approval.png", name),
        format!("plots/{} Support.png", name),
        format!("points/{} Approval.csv", name),
        format!("points/{} Support.csv", name),
    );
    let app_grid = BitMapBackend::new(&app_plot_png, (600, 400)).into_drawing_area();
    app_grid.fill(&WHITE).unwrap();
    let sup_grid = BitMapBackend::new(&sup_plot_png, (600, 400)).into_drawing_area();
    sup_grid.fill(&WHITE).unwrap();
    let hours = 24 * days;
    let mut app_plot = ChartBuilder::on(&app_grid)
        .caption(
            &format!("{} Approval, TrackID #{}", name, id),
            ("sans-serif", 30),
        )
        .margin(5)
        .set_left_and_bottom_label_area_size(40)
        .build_cartesian_2d(0..hours + 1, 0..100)
        .unwrap();
    let mut sup_plot = ChartBuilder::on(&sup_grid)
        .caption(
            &format!("{} Support, TrackID #{}", name, id),
            ("sans-serif", 30),
        )
        .margin(5)
        .set_left_and_bottom_label_area_size(40)
        .build_cartesian_2d(0..hours + 1, 0..100)
        .unwrap();
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
    let curve_points = |crv, pts| {
        (0..=pts).map(move |x| {
            (
                x,
                perbill_to_percent_coordinate(threshold(crv, Perbill::from_rational(x, pts))),
            )
        })
    };
    app_plot
        .draw_series(LineSeries::new(curve_points(approval_curve, hours), &RED))
        .unwrap();
    sup_plot
        .draw_series(LineSeries::new(curve_points(support_curve, hours), &RED))
        .unwrap();
    let approval_curve_points: Vec<(u32, i32)> = curve_points(approval_curve, hours).collect();
    write_curve_points_csv(app_points_csv, approval_curve_points.clone());
    let support_curve_points: Vec<(u32, i32)> = curve_points(support_curve, hours).collect();
    write_curve_points_csv(sup_points_csv, support_curve_points.clone());
    (approval_curve_points, support_curve_points)
}

/// Write curve points to file
fn write_curve_points_csv(file: String, points: Vec<(u32, i32)>) {
    let mut file = File::create(file).unwrap();
    for (x, y) in points {
        file.write_all(format!("{}, {}\n", x, y).as_bytes())
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

// TODO: expose in substrate
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
