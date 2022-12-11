use std::fs;

use gov2_curve_plotter::*;

// Tmp config for all curves
const WRITE_TO_CSV: bool = true;
const PLOT: bool = true;
const PLOT_COMPARISON: bool = true;
const OVERWRITE_PREVIOUS_DATA_ON_RUN: bool = true;

fn decision_period(unit: Time, length: u32) -> TimeLength {
    let length = match unit {
        Time::Hour => (length / DAYS) * 24,
        Time::Minute => (length / DAYS) * 24 * 60,
        Time::Second => (length / DAYS) * 24 * 60 * 60,
    };
    TimeLength { unit, length }
}

fn main() {
    if OVERWRITE_PREVIOUS_DATA_ON_RUN {
        fs::remove_dir_all("data").ok();
        fs::create_dir_all("data").expect("data/");
        fs::create_dir_all("data/points").expect("data/points/");
        fs::create_dir_all("data/plots").expect("data/plots/");
    }
    let (mut approval_curves, mut support_curves) = (Curves::new(), Curves::new());
    for (track_id, track) in TRACKS_DATA {
        let time = decision_period(Time::Hour, track.decision_period);
        let (approval_curve, support_curve) = (
            CurvePoints::new(
                CurveType::Approval,
                track_id,
                track.name.to_string(),
                time,
                &track.min_approval,
            ),
            CurvePoints::new(
                CurveType::Support,
                track_id,
                track.name.to_string(),
                time,
                &track.min_support,
            ),
        );
        if WRITE_TO_CSV {
            approval_curve.write_to_csv();
            support_curve.write_to_csv();
        }
        if PLOT {
            approval_curve.plot();
            support_curve.plot();
        }
        approval_curves.push(approval_curve);
        support_curves.push(support_curve);
    }
    if PLOT_COMPARISON {
        plot_approval_curves(approval_curves);
        plot_support_curves(support_curves);
    }
}
