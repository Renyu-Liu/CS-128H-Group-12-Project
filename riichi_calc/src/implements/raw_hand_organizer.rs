// Import all the new modular types
use super::types::{
    game::{AgariType, GameContext, PlayerContext},
    hand::{AgariHand, HandOrganization, Machi, Mentsu, MentsuType},
    // Import centralized helper functions
    tiles::{index_to_tile, tile_to_index, Hai, Suhai},
    input::{UserInput},
};
// Used for converting Vec<Mentsu> to [Mentsu; 4]
use std::convert::TryInto;

// === Input Validation Module ===
mod input_validator {
    use super::*;

    /// Checks for logical conflicts in declared game state yaku.
    fn validate_game_state(
        p: &PlayerContext,
        g: &GameContext,
        a: AgariType,
        input: &UserInput,
    ) -> Result<(), &'static str> {
        // Riichi conflicts
        if p.is_daburu_riichi && p.is_riichi {
            return Err("Invalid state: Cannot be both Riichi and Daburu Riichi.");
        }
        if p.is_ippatsu && !(p.is_riichi || p.is_daburu_riichi) {
            return Err("Invalid state: Ippatsu requires Riichi or Daburu Riichi.");
        }

        // Menzen (Concealed) conflicts
        if p.is_menzen && !input.open_melds.is_empty() {
            return Err("Invalid state: Hand is declared menzen but has open melds.");
        }
        // --- LOGIC FIX: Allow menzen=false with no open melds ---
        // This is a valid state (e.g., fully concealed hand, but
        // player chooses not to declare riichi, or a hand that
        // cannot be menzen like Chiitoitsu, but still has no "open" melds)
        // A player can have a concealed hand but not be "menzen"
        // if they don't win on tsumo.
        // The *real* conflict is `is_menzen = true` with open melds.
        // We will trust the `is_menzen` flag from the user, as it
        // affects yaku like Pinfu, Tsumo, etc.
        /*
        if !p.is_menzen && input.open_melds.is_empty() {
            // This check is problematic. A concealed hand won on Ron
            // might not be considered "menzen" for scoring Tsumo,
            // but it is still concealed for han calculation.
            // We'll trust the user's `is_menzen` flag.
        }
        */

        // Tsumo/Ron conflicts
        if g.is_haitei && a == AgariType::Ron {
            return Err("Invalid state: Haitei (last draw) cannot be a Ron win.");
        }
        if g.is_houtei && a == AgariType::Tsumo {
            return Err("Invalid state: Houtei (last discard) cannot be a Tsumo win.");
        }
        if g.is_haitei && g.is_houtei {
            return Err("Invalid state: Cannot be both Haitei and Houtei.");
        }
        if g.is_rinshan && a == AgariType::Ron {
            return Err("Invalid state: Rinshan (kan draw) cannot be a Ron win.");
        }
        if g.is_chankan && a == AgariType::Tsumo {
            return Err("Invalid state: Chankan (robbing kan) cannot be a Tsumo win.");
        }

        // Yakuman state conflicts
        if g.is_tenhou {
            if !p.is_oya {
                return Err("Invalid state: Tenhou requires player to be Oya (dealer).");
            }
            if a != AgariType::Tsumo {
                return Err("Invalid state: Tenhou must be a Tsumo win.");
            }
            if !input.open_melds.is_empty() || !input.closed_kans.is_empty() {
                return Err("Invalid state: Tenhou cannot have any calls (no open melds or kans).");
            }
        }
        if g.is_chiihou {
            if p.is_oya {
                return Err("Invalid state: Chiihou requires player to be non-Oya.");
            }
            if a != AgariType::Tsumo {
                return Err("Invalid state: Chiihou must be a Tsumo win.");
            }
            if !input.open_melds.is_empty() || !input.closed_kans.is_empty() {
                return Err("Invalid state: Chiihou cannot have any calls (no open melds or kans).");
            }
        }
        if g.is_renhou && a != AgariType::Ron {
            return Err("Invalid state: Renhou must be a Ron win.");
        }

        Ok(())
    }

    /// Checks for invalid hand composition (tile counts, meld counts, etc.)
    fn validate_hand_composition(
        input: &UserInput,
        master_counts: &[u8; 34],
    ) -> Result<(), &'static str> {
        // Check 1: Total Meld Count
        if input.closed_kans.len() + input.open_melds.len() > 4 {
            return Err("Invalid hand: More than 4 total melds (kans + open melds) declared.");
        }

        // Check 2: Total Tile Count based on melds
        let total_kans = input.closed_kans.len()
            + input
                .open_melds
                .iter()
                .filter(|m| m.mentsu_type == MentsuType::Kantsu)
                .count();

        // This calculation assumes the input is for a *complete* hand (4 melds + 1 pair)
        // Formula: (total_kans * 4) + ((4 - total_kans) * 3) + 2
        
        let expected_tiles = (total_kans * 4) + ((4 - total_kans) * 3) + 2;

        // --- LOGIC FIX: Allow for Kokushi/Chiitoitsu (14 tiles) ---
        // The standard hand check is only valid if the hand IS standard.
        // Kokushi and Chiitoitsu always have 14 tiles and 0 kans.
        let hand_len = input.hand_tiles.len();
        if hand_len == 14 && total_kans == 0 {
            // This could be a 0-kan standard hand, OR Chiitoitsu/Kokushi.
            // This is valid.
        } else if hand_len != expected_tiles {
             // It's not a 14-tile/0-kan hand, so it MUST match
             // the kan-based count.
            let err_msg = "Invalid hand: Tile count does not match declared kans. (Expected 14 for 0 kans, 15 for 1 kan, 16 for 2, 17 for 3, 18 for 4).";
            return Err(err_msg);
        }

        // Check 3: Winning Tile Presence
        if !input.hand_tiles.contains(&input.winning_tile) {
            return Err("Invalid input: Winning tile is not present in the list of hand tiles.");
        }

        // Check 4: Max 4 of any tile (checked from master_counts)
        if master_counts.iter().any(|&count| count > 4) {
            return Err("Invalid hand: Contains 5 or more of a single tile type.");
        }

        // Check 5: Akadora counts (using your new field)
        let num_5m = master_counts[tile_to_index(&Hai::Suhai(5, Suhai::Manzu))];
        let num_5p = master_counts[tile_to_index(&Hai::Suhai(5, Suhai::Pinzu))];
        let num_5s = master_counts[tile_to_index(&Hai::Suhai(5, Suhai::Souzu))];
        let total_fives = num_5m + num_5p + num_5s;

        if input.game_context.num_akadora > total_fives {
            return Err(
                "Invalid input: Number of akadora exceeds the total number of '5' tiles in the hand.",
            );
        }
        
        // Sanity check: most rulesets have 3 or 4 red fives *total in the deck*.
        // A hand having more than 4 is impossible.
        if input.game_context.num_akadora > 4 {
            return Err("Invalid input: Number of akadora cannot be greater than 4.");
        }

        Ok(())
    }

    /// Runs all validation checks.
    pub fn validate_input(input: &UserInput, master_counts: &[u8; 34]) -> Result<(), &'static str> {
        // Validate game state conflicts first
        validate_game_state(
            &input.player_context,
            &input.game_context,
            input.agari_type,
            input,
        )?;
        
        // Then validate hand composition
        validate_hand_composition(input, master_counts)?;
        
        Ok(())
    }
}

// === Recursive Parsing Logic ===
mod recursive_parser {
    use super::*;

    /// Recursively finds melds from a tile-count array.
    pub fn find_mentsu_recursive(counts: &mut [u8; 34], mentsu: &mut Vec<Mentsu>) -> bool {
        let mut i = 0;
        while i < 34 && counts[i] == 0 {
            i += 1;
        }
        if i == 34 {
            return true;
        } // Success: all tiles used up

        // --- Try to form a Triplet (Koutsu) ---
        if counts[i] >= 3 {
            let tile = index_to_tile(i);
            counts[i] -= 3;
            mentsu.push(Mentsu {
                mentsu_type: MentsuType::Koutsu,
                is_minchou: false, // is_open
                tiles: [tile, tile, tile, tile], // 4th tile is ignored
            });

            if find_mentsu_recursive(counts, mentsu) {
                return true;
            }

            // Backtrack
            mentsu.pop();
            counts[i] += 3;
        }

        // --- Try to form a Sequence (Shuntsu) ---
        // Check i < 27 (not Jihai)
        // Check (i % 9) < 7 (not 8s or 9s)
        if i < 27 && (i % 9) < 7 && counts[i] > 0 && counts[i + 1] > 0 && counts[i + 2] > 0 {
            let tile1 = index_to_tile(i);
            let tile2 = index_to_tile(i + 1);
            let tile3 = index_to_tile(i + 2);

            counts[i] -= 1;
            counts[i + 1] -= 1;
            counts[i + 2] -= 1;
            mentsu.push(Mentsu {
                mentsu_type: MentsuType::Shuntsu,
                is_minchou: false,
                tiles: [tile1, tile2, tile3, tile3],
            });

            if find_mentsu_recursive(counts, mentsu) {
                return true;
            }

            // Backtrack
            mentsu.pop();
            counts[i] += 1;
            counts[i + 1] += 1;
            counts[i + 2] += 1;
        }

        false
    }
}

// === Wait Type Analysis Logic ===
mod wait_analyzer {
    use super::*;

    fn mentsu_contains_tile(mentsu: &Mentsu, tile: &Hai) -> bool {
        match mentsu.mentsu_type {
            MentsuType::Koutsu | MentsuType::Kantsu => mentsu.tiles[0] == *tile,
            MentsuType::Shuntsu => {
                mentsu.tiles[0] == *tile || mentsu.tiles[1] == *tile || mentsu.tiles[2] == *tile
            }
        }
    }

    pub fn determine_wait_type(
        mentsu: &[Mentsu; 4],
        atama: (Hai, Hai), 
        agari_hai: Hai,    
    ) -> Machi {
        if agari_hai == atama.0 {
            return Machi::Tanki;
        }

        let winning_meld = mentsu
            .iter()
            .find(|m| mentsu_contains_tile(m, &agari_hai))
            .expect("Winning tile not in pair or melds. Invalid hand.");

        match winning_meld.mentsu_type {
            MentsuType::Koutsu | MentsuType::Kantsu => Machi::Shanpon,
            MentsuType::Shuntsu => {
                let t1 = winning_meld.tiles[0];
                let t2 = winning_meld.tiles[1];
                let t3 = winning_meld.tiles[2];

                if agari_hai == t2 {
                    // e.g., 4-6 waiting on 5 (tile 2)
                    Machi::Kanchan
                } else if agari_hai == t1 {
                    // e.g., 8-9 waiting on 7 (tile 1)
                    if tile_to_index(&t3) % 9 == 8 {
                        // t3 is a 9 (e.g., 7-8-9)
                        Machi::Penchan
                    } else {
                        // e.g., 5-6 waiting on 4 (tile 1)
                        Machi::Ryanmen
                    }
                } else if agari_hai == t3 {
                     // e.g., 1-2 waiting on 3 (tile 3)
                    if tile_to_index(&t1) % 9 == 0 {
                         // t1 is a 1 (e.g., 1-2-3)
                        Machi::Penchan
                    } else {
                        // e.g., 5-6 waiting on 7 (tile 3)
                        Machi::Ryanmen
                    }
                } else {
                    unreachable!("Winning tile in sequence but not t1, t2, or t3");
                }
            }
        }
    }
}

// === Public Function ===

/// Organizes a raw hand from `UserInput` into a standard 4-meld, 1-pair structure
/// or flags it as irregular for special yaku checking (Chiitoitsu, Kokushi).
///
/// # Arguments
/// * `input` - The `UserInput` struct containing all tiles, meld info, and context.
pub fn organize_hand(input: &UserInput) -> Result<HandOrganization, &'static str> {
    
    // 1. Create a count of ALL tiles provided by the user (14-18 tiles)
    let mut master_counts = [0u8; 34];
    for tile in &input.hand_tiles {
        master_counts[tile_to_index(tile)] += 1;
    }

    // --- NEW: Run all validation checks FIRST ---
    // This will return an Err(&'static str) if any rule is violated.
    input_validator::validate_input(input, &master_counts)?;
    // --- End of new validation block ---


    // 2. Create counts for the *concealed* part of the hand
    //    We start with all tiles and subtract the known melds.
    let mut concealed_counts = master_counts;
    let mut final_mentsu: Vec<Mentsu> = Vec::with_capacity(4);

    // 3. Process and subtract Closed Kans (Ankan)
    for rep_tile in &input.closed_kans {
        let kan_tile = *rep_tile;
        let index = tile_to_index(&kan_tile);

        if concealed_counts[index] < 4 {
            // This check is still good, acting as a failsafe
            return Err("Invalid input: declared closed kan not present in hand tiles.");
        }
        concealed_counts[index] -= 4;
        final_mentsu.push(Mentsu {
            mentsu_type: MentsuType::Kantsu,
            is_minchou: false, // Ankan is not "open"
            tiles: [kan_tile, kan_tile, kan_tile, kan_tile],
        });
    }

    // 4. Process and subtract Open Melds (Chi, Pon, Daiminkan, Shouminkan)
    for meld in &input.open_melds {
        let rep_tile = meld.representative_tile;
        let index = tile_to_index(&rep_tile);

        match meld.mentsu_type {
            MentsuType::Koutsu => {
                if concealed_counts[index] < 3 {
                    return Err("Invalid input: declared Pon not present in hand tiles.");
                }
                concealed_counts[index] -= 3;
                final_mentsu.push(Mentsu {
                    mentsu_type: MentsuType::Koutsu,
                    is_minchou: true, 
                    tiles: [rep_tile, rep_tile, rep_tile, rep_tile], 
                });
            }
            MentsuType::Kantsu => {
                if concealed_counts[index] < 4 {
                    return Err("Invalid input: declared open Kan not present in hand tiles.");
                }
                concealed_counts[index] -= 4;
                final_mentsu.push(Mentsu {
                    mentsu_type: MentsuType::Kantsu,
                    is_minchou: true, 
                    tiles: [rep_tile, rep_tile, rep_tile, rep_tile],
                });
            }
            MentsuType::Shuntsu => {
                let index1 = index;
                let index2 = index1 + 1;
                let index3 = index1 + 2;

                if index1 >= 27 || (index1 % 9) >= 7 {
                    return Err("Invalid representative tile for Chi (must be 1-7 of a suit).");
                }
                if concealed_counts[index1] < 1
                    || concealed_counts[index2] < 1
                    || concealed_counts[index3] < 1
                {
                    return Err("Invalid input: declared Chi not present in hand tiles.");
                }

                concealed_counts[index1] -= 1;
                concealed_counts[index2] -= 1;
                concealed_counts[index3] -= 1;

                let t1 = rep_tile;
                let t2 = index_to_tile(index2);
                let t3 = index_to_tile(index3);
                final_mentsu.push(Mentsu {
                    mentsu_type: MentsuType::Shuntsu,
                    is_minchou: true, 
                    tiles: [t1, t2, t3, t3], 
                });
            }
        }
    }

    // 5. Determine how many closed melds we still need to find
    let mentsu_needed = 4 - final_mentsu.len();
    let agari_hai = input.winning_tile;

    // --- Case A: 4 known melds (e.g., Hadaka Tanki / Naked Wait) ---
    if mentsu_needed == 0 {
        for i in 0..34 {
            if concealed_counts[i] == 2 {
                let pair_tile = index_to_tile(i);
                let atama = (pair_tile, pair_tile);

                let mentsu_array: [Mentsu; 4] = final_mentsu
                    .try_into()
                    .expect("Hand parsing logic error: final_mentsu length not 4");

                let agari_hand = AgariHand {
                    mentsu: mentsu_array,
                    atama,
                    agari_hai,
                    machi: Machi::Tanki, 
                };

                return Ok(HandOrganization::YonmentsuIchiatama(agari_hand));
            }
        }
        // Check for 14-tile hand that is NOT Chiitoitsu/Kokushi
        // This case is only possible if validation is wrong.
        if input.hand_tiles.len() == 14 {
             // Fall through to Irregular check
        } else {
            return Err("Invalid hand: 4 open melds but no pair found.");
        }
    }

    // --- Case B: 0-3 known melds (Standard Hand Check) ---
    for i in 0..34 {
        if concealed_counts[i] >= 2 {
            let mut temp_counts = concealed_counts; 
            temp_counts[i] -= 2;
            let atama = (index_to_tile(i), index_to_tile(i));
            let mut closed_mentsu: Vec<Mentsu> = Vec::with_capacity(mentsu_needed);

            if recursive_parser::find_mentsu_recursive(&mut temp_counts, &mut closed_mentsu) {
                if closed_mentsu.len() == mentsu_needed {
                    final_mentsu.append(&mut closed_mentsu);

                    let mentsu_array: [Mentsu; 4] = final_mentsu
                        .try_into()
                        .expect("Hand parsing logic error: final_mentsu length not 4");

                    let machi =
                        wait_analyzer::determine_wait_type(&mentsu_array, atama, agari_hai);

                    let agari_hand = AgariHand {
                        mentsu: mentsu_array,
                        atama,
                        agari_hai,
                        machi,
                    };

                    return Ok(HandOrganization::YonmentsuIchiatama(agari_hand));
                }
            }
        }
    }

    // --- FAILURE ---
    // If we are here, the 4-meld-1-pair parse failed.
    // This means the hand is either irregular (Chiitoitsu, Kokushi) or invalid.
    
    // We've already checked for tile count/composition errors, so if
    // it's irregular, it's now the yaku_checker's job to validate
    // it as Kokushi or Chiitoitsu.
    Ok(HandOrganization::Irregular {
        counts: master_counts,
        agari_hai,
    })
}