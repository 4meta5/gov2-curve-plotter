// TODO: replace this with something that parses this from the moonbeam repo
// instead of hardcoded as is

/// Balance of an account.
pub type Balance = u128;
/// An index to a block.
pub type BlockNumber = u32;

pub const MILLISECS_PER_BLOCK: u64 = 12000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const WEEKS: BlockNumber = DAYS * 7;

const fn percent(x: i32) -> sp_runtime::FixedI64 {
    sp_runtime::FixedI64::from_rational(x as u128, 100)
}
use pallet_referenda::Curve;
pub const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 8] = [
    (
        0,
        pallet_referenda::TrackInfo {
            // Name of this track.
            name: "root",
            // A limit for the number of referenda on this track that can be being decided at once.
            // For Root origin this should generally be just one.
            max_deciding: 1,
            // Amount that must be placed on deposit before a decision can be made.
            decision_deposit: 100,
            // Amount of time this must be submitted for before a decision can be made.
            prepare_period: 3 * HOURS,
            // Amount of time that a decision may take to be approved prior to cancellation.
            decision_period: 14 * DAYS,
            // Amount of time that the approval criteria must hold before it can be approved.
            confirm_period: 3 * HOURS,
            // Minimum amount of time that an approved proposal must be in the dispatch queue.
            min_enactment_period: 3 * HOURS,
            // Minimum aye votes as percentage of overall conviction-weighted votes needed for
            // approval as a function of time into decision period.
            min_approval: Curve::make_reciprocal(4, 14, percent(80), percent(50), percent(100)),
            // Minimum pre-conviction aye-votes ("support") as percentage of overall population that
            // is needed for approval as a function of time into decision period.
            min_support: Curve::make_linear(14, 14, percent(0), percent(50)),
        },
    ),
    (
        1,
        pallet_referenda::TrackInfo {
            name: "whitelisted_caller",
            max_deciding: 10,
            decision_deposit: 10,
            prepare_period: 30 * MINUTES,
            decision_period: 14 * DAYS,
            confirm_period: 10 * MINUTES,
            min_enactment_period: 30 * MINUTES,
            min_approval: Curve::make_reciprocal(
                1,
                14 * 24,
                percent(96),
                percent(50),
                percent(100),
            ),
            min_support: Curve::make_reciprocal(1, 14 * 24, percent(4), percent(2), percent(50)),
        },
    ),
    (
        10,
        pallet_referenda::TrackInfo {
            name: "treasurer",
            max_deciding: 1,
            decision_deposit: 10,
            prepare_period: 1 * DAYS,
            decision_period: 14 * DAYS,
            confirm_period: 2 * DAYS,
            min_enactment_period: 2 * DAYS,
            min_approval: Curve::make_linear(14, 14, percent(50), percent(100)),
            min_support: Curve::make_reciprocal(10, 14, percent(10), percent(0), percent(50)),
        },
    ),
    (
        11,
        pallet_referenda::TrackInfo {
            name: "referendum_canceller",
            max_deciding: 100,
            decision_deposit: 5,
            prepare_period: 4,
            decision_period: 14 * DAYS,
            confirm_period: 1 * DAYS,
            min_enactment_period: 10 * MINUTES,
            min_approval: Curve::make_reciprocal(1, 14, percent(96), percent(50), percent(100)),
            min_support: Curve::make_reciprocal(1, 14, percent(1), percent(0), percent(50)),
        },
    ),
    (
        12,
        pallet_referenda::TrackInfo {
            name: "referendum_killer",
            max_deciding: 100,
            decision_deposit: 5,
            prepare_period: 4,
            decision_period: 14 * DAYS,
            confirm_period: 1 * DAYS,
            min_enactment_period: 10 * MINUTES,
            min_approval: Curve::make_reciprocal(1, 14, percent(96), percent(50), percent(100)),
            min_support: Curve::make_reciprocal(7, 14, percent(1), percent(0), percent(10)),
        },
    ),
    (
        13,
        pallet_referenda::TrackInfo {
            name: "small_spender",
            max_deciding: 5,
            decision_deposit: 300,
            prepare_period: 4,
            decision_period: 14 * DAYS,
            confirm_period: 12 * HOURS,
            min_enactment_period: 1 * DAYS,
            min_approval: Curve::make_linear(8, 14, percent(50), percent(100)),
            min_support: Curve::make_reciprocal(2, 14, percent(1), percent(0), percent(10)),
        },
    ),
    (
        14,
        pallet_referenda::TrackInfo {
            name: "medium_spender",
            max_deciding: 5,
            decision_deposit: 3000,
            prepare_period: 4,
            decision_period: 14 * DAYS,
            confirm_period: 24 * HOURS,
            min_enactment_period: 1 * DAYS,
            min_approval: Curve::make_linear(10, 14, percent(50), percent(100)),
            min_support: Curve::make_reciprocal(4, 14, percent(1), percent(0), percent(10)),
        },
    ),
    (
        15,
        pallet_referenda::TrackInfo {
            name: "big_spender",
            max_deciding: 5,
            decision_deposit: 30,
            prepare_period: 4,
            decision_period: 14 * DAYS,
            confirm_period: 48 * HOURS,
            min_enactment_period: 1 * DAYS,
            min_approval: Curve::make_linear(14, 14, percent(50), percent(100)),
            min_support: Curve::make_reciprocal(8, 14, percent(1), percent(0), percent(10)),
        },
    ),
];
