use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::TracksInfo;

mod curve;
use curve::*;

fn main() {
    // TODO: if directories don't exist, make them(in the functions)
    let (mut approval_curves, mut support_curves) = (Curves::new(), Curves::new());
    for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
        approval_curves.push(plot_curve(
            track.name.to_string(),
            *track_id,
            CurveType::Approval,
            &track.min_approval,
            (track.decision_period / DAYS) * 24,
        ));
        support_curves.push(plot_curve(
            track.name.to_string(),
            *track_id,
            CurveType::Support,
            &track.min_support,
            (track.decision_period / DAYS) * 24,
        ));
    }
    plot_approval_curves(approval_curves);
    plot_support_curves(support_curves);
}
