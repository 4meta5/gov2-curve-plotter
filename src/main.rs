use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::TracksInfo;

mod curve;
use curve::{plot_curve, CurveType};

fn main() {
    for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
        let decision_period_days = track.decision_period / DAYS;
        plot_curve(
            CurveType::Approval,
            track.name.to_string(),
            *track_id,
            &track.min_approval,
            decision_period_days,
        );
        plot_curve(
            CurveType::Support,
            track.name.to_string(),
            *track_id,
            &track.min_support,
            decision_period_days,
        );
    }
}
