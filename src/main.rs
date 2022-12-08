use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::TracksInfo;

mod curve;
use curve::*;

// Tmp config for all curves
const WRITE_TO_CSV: bool = true;
const PLOT: bool = true;
const PLOT_COMPARISON: bool = true;

fn main() {
    // TODO: if directories don't exist, make them(in the functions)
    let (mut approval_curves, mut support_curves) = (Curves::new(), Curves::new());
    for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
        let time = TimeLength {
            unit: Time::Hour,
            length: (track.decision_period / DAYS) * 24,
        };
        let (approval_curve, support_curve) = (
            CurvePoints::new(*track_id, track.name.to_string(), time, &track.min_approval),
            CurvePoints::new(*track_id, track.name.to_string(), time, &track.min_support),
        );
        if WRITE_TO_CSV {
            approval_curve.write_to_csv(CurveType::Approval);
            support_curve.write_to_csv(CurveType::Support);
        }
        if PLOT {
            approval_curve.plot(CurveType::Approval);
            support_curve.plot(CurveType::Support);
        }
        approval_curves.push(approval_curve);
        support_curves.push(support_curve);
    }
    if PLOT_COMPARISON {
        plot_approval_curves(approval_curves);
        plot_support_curves(support_curves);
    }
}
