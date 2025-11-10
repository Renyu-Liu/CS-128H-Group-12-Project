use super::types::*;
// We need the tile_to_index and index_to_tile functions from the organizer
use super::raw_hand_organizer::helpers;

/// Checks if the hand is a valid Seven Pairs (Chiitoitsu).
/// This must be called ONLY on a closed hand.
fn check_seven_pairs(counts: &[u8; 34], winning_tile: Tile) -> Option<HandStructure> {
    // 1. Check if there are exactly 7 pairs.
    let pair_count = counts.iter().filter(|&&c| c == 2).count();
    
    if pair_count == 7 {
        // 2. Hand has 7 pairs. Now, construct the list of pairs.
        let mut pairs = [winning_tile; 7]; // Placeholder, will be overwritten
        let mut pair_idx = 0;
        for i in 0..34 {
            if counts[i] == 2 {
                pairs[pair_idx] = helpers::index_to_tile(i);
                pair_idx += 1;
            }
        }
        
        // A 7-pairs hand is always a pair wait (tanki machi).
        return Some(HandStructure::SevenPairs {
            pairs,
            winning_tile,
            wait_type: WaitType::Pair,
        });
    }

    None
}

/// Checks if the hand is a valid Thirteen Orphans (Kokushi Musou).
/// This must be called ONLY on a closed hand.
fn check_thirteen_orphans(counts: &[u8; 34], winning_tile: Tile) -> Option<HandStructure> {
    // 1. Define the 13 terminals and honors indices
    const KOKUSHI_INDICES: [usize; 13] = [
        0, 8, // 1m, 9m
        9, 17, // 1p, 9p
        18, 26, // 1s, 9s
        27, 28, 29, 30, // East, South, West, North
        31, 32, 33, // White, Green, Red
    ];

    // 2. Check if all 13 tiles are present (at least one of each)
    let has_all_13 = KOKUSHI_INDICES.iter().all(|&i| counts[i] >= 1);
    if !has_all_13 {
        return None;
    }

    // 3. Find the tile that forms the pair
    let mut pair_tile = None;
    for &i in &KOKUSHI_INDICES {
        if counts[i] == 2 {
            pair_tile = Some(helpers::index_to_tile(i));
            break;
        }
    }

    // 4. If we found a pair, it's a valid Kokushi
    if let Some(pair_tile) = pair_tile {
        // Check if the winning tile is part of the pair (13-sided wait)
        // or one of the singles (single wait).
        let wait_type = if winning_tile == pair_tile {
            // This is the "13-sided" wait
            WaitType::KokushiThirteenSided
        } else {
            // This is the "single" wait
            WaitType::KokushiSingle
        };
        
        return Some(HandStructure::ThirteenOrphans {
            pair_tile,
            winning_tile,
            wait_type,
        });
    }

    None
}

/// Public function to check all special hand types.
/// This function assumes the hand is CLOSED.
pub fn check_special_hands(raw_hand: &RawHandInput, counts: &[u8; 34]) -> Option<HandStructure> {
    // A hand cannot be open and be a special hand (7-pairs/13-orphans)
    if !raw_hand.open_melds.is_empty() {
        return None;
    }

    // Try checking for 7 pairs
    if let Some(hand_structure) = check_seven_pairs(counts, raw_hand.winning_tile) {
        return Some(hand_structure);
    }

    // Try checking for 13 orphans
    if let Some(hand_structure) = check_thirteen_orphans(counts, raw_hand.winning_tile) {
        return Some(hand_structure);
    }

    // No special hands found
    None
}