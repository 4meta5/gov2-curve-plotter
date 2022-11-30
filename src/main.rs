use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::TracksInfo;

mod curve;
use curve::*;

fn main() {
    // graph all approval/support labeled by "name track ID"
    let (mut approval_curves, mut support_curves) = (Curves::new(), Curves::new());
    for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
        let (approval_curve_points, support_curve_points) = plot_track_curves(
            track.name.to_string(),
            *track_id,
            &track.min_approval,
            &track.min_support,
            track.decision_period / DAYS,
        );
        approval_curves.push(CurveInfo {
            track_id: *track_id,
            name: track.name.to_string(),
            points: approval_curve_points,
        });
        support_curves.push(CurveInfo {
            track_id: *track_id,
            name: track.name.to_string(),
            points: support_curve_points,
        });
    }
    // plot approval curves altogether
    // plot support curves altogether
}
