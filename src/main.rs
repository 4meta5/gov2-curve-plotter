use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::TracksInfo;

mod curve;
use curve::*;

fn main() {
    // TODO: if directories don't exist, make them(in the functions)
    let (mut approval_curves, mut support_curves) = (Curves::new(), Curves::new());
    for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
        approval_curves.push(CurveInfo {
            track_id: *track_id,
            name: track.name.to_string(),
            points: plot_curve(
                track.name.to_string(),
                *track_id,
                CurveType::Approval,
                &track.min_approval,
                (track.decision_period / DAYS) * 24,
                Vec::new(),
                true,
            ),
        });
        support_curves.push(CurveInfo {
            track_id: *track_id,
            name: track.name.to_string(),
            points: plot_curve(
                track.name.to_string(),
                *track_id,
                CurveType::Support,
                &track.min_support,
                (track.decision_period / DAYS) * 24,
                Vec::new(),
                true,
            ),
        });
    }
    plot_approval_curves(approval_curves);
    plot_support_curves(support_curves);
}
