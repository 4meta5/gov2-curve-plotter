use moonbase_runtime::{governance::TracksInfo as MoonbaseTracks, Balance, BlockNumber, DAYS};
use pallet_referenda::TracksInfo;

mod curve;
use curve::plot_track_curves;

fn main() {
    for (track_id, track) in <MoonbaseTracks as TracksInfo<Balance, BlockNumber>>::tracks() {
        plot_track_curves(
            track.name.to_string(),
            *track_id,
            &track.min_approval,
            &track.min_support,
            track.decision_period / DAYS,
        );
    }
}
