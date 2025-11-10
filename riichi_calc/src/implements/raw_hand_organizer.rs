use super::types::*;
use super::special_yaku_checker; // <-- Import the new module

// === Private Helper Functions ===
// Encapsulated in a public module so other files can use it
pub mod helpers {
    use super::*;

    /// Converts a Tile enum to its corresponding index (0-33).
    pub fn tile_to_index(tile: &Tile) -> usize {
        match tile {
            Tile::Number(n, Suit::Man) => (n - 1) as usize,       // 0-8
            Tile::Number(n, Suit::Pin) => (n - 1) as usize + 9,  // 9-17
            Tile::Number(n, Suit::Sou) => (n - 1) as usize + 18, // 18-26
            Tile::Honor(Honor::Wind(Wind::East)) => 27,
            Tile::Honor(Honor::Wind(Wind::South)) => 28,
            Tile::Honor(Honor::Wind(Wind::West)) => 29,
            Tile::Honor(Honor::Wind(Wind::North)) => 30,
            Tile::Honor(Honor::Dragon(Dragon::White)) => 31,
            Tile::Honor(Honor::Dragon(Dragon::Green)) => 32,
            Tile::Honor(Honor::Dragon(Dragon::Red)) => 33,
        }
    }

    /// Converts an index (0-33) back into a Tile.
    pub fn index_to_tile(index: usize) -> Tile {
        match index {
            0..=8 => Tile::Number((index + 1) as u8, Suit::Man),
            9..=17 => Tile::Number(((index - 9) + 1) as u8, Suit::Pin),
            18..=26 => Tile::Number(((index - 18) + 1) as u8, Suit::Sou),
            27 => Tile::Honor(Honor::Wind(Wind::East)),
            28 => Tile::Honor(Honor::Wind(Wind::South)),
            29 => Tile::Honor(Honor::Wind(Wind::West)),
            30 => Tile::Honor(Honor::Wind(Wind::North)),
            31 => Tile::Honor(Honor::Dragon(Dragon::White)),
            32 => Tile::Honor(Honor::Dragon(Dragon::Green)),
            33 => Tile::Honor(Honor::Dragon(Dragon::Red)),
            _ => panic!("Invalid tile index: {}", index),
        }
    }
}

// === Recursive Parsing Logic ===
mod recursive_parser {
    use super::*;
    // ... (This module remains unchanged) ...
    /// Recursively finds melds from a tile-count array.
    pub fn find_melds_recursive(counts: &mut [u8; 34], melds: &mut Vec<Meld>) -> bool {
        let mut i = 0;
        while i < 34 && counts[i] == 0 {
            i += 1;
        }
        if i == 34 { return true; }

        // --- Try to form a Triplet (Koutsu) ---
        if counts[i] >= 3 {
            let tile = helpers::index_to_tile(i);
            counts[i] -= 3;
            melds.push(Meld {
                meld_type: MeldType::Triplet,
                is_open: false,
                tiles: [tile, tile, tile, tile], // 4th tile is ignored
            });

            if find_melds_recursive(counts, melds) { return true; }

            melds.pop();
            counts[i] += 3;
        }

        // --- Try to form a Sequence (Shuntsu) ---
        if i < 27 && (i % 9) < 7 && counts[i] > 0 && counts[i + 1] > 0 && counts[i + 2] > 0 {
            let tile1 = helpers::index_to_tile(i);
            let tile2 = helpers::index_to_tile(i + 1);
            let tile3 = helpers::index_to_tile(i + 2);

            counts[i] -= 1;
            counts[i + 1] -= 1;
            counts[i + 2] -= 1;
            melds.push(Meld {
                meld_type: MeldType::Sequence,
                is_open: false,
                tiles: [tile1, tile2, tile3, tile3], // Store sorted, 4th is ignored
            });

            if find_melds_recursive(counts, melds) { return true; }

            melds.pop();
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
    // ... (This module remains unchanged) ...
    /// Checks if a meld contains a specific tile.
    fn meld_contains_tile(meld: &Meld, tile: &Tile) -> bool {
        match meld.meld_type {
            MeldType::Triplet | MeldType::Kan => meld.tiles[0] == *tile,
            MeldType::Sequence => {
                meld.tiles[0] == *tile || meld.tiles[1] == *tile || meld.tiles[2] == *tile
            }
        }
    }

    /// Analyzes the completed hand to determine the wait type.
    pub fn determine_wait_type(
        melds: &[Meld; 4],
        pair: (Tile, Tile),
        winning_tile: Tile,
    ) -> WaitType {
        // 1. Check for Pair Wait (Tanki)
        if winning_tile == pair.0 {
            return WaitType::Pair;
        }

        let winning_meld = melds
            .iter()
            .find(|m| meld_contains_tile(m, &winning_tile))
            .expect("Winning tile not in pair or melds. Invalid hand.");

        match winning_meld.meld_type {
            MeldType::Triplet | MeldType::Kan => WaitType::Single,
            MeldType::Sequence => {
                let t1 = winning_meld.tiles[0];
                let t2 = winning_meld.tiles[1];
                let t3 = winning_meld.tiles[2];

                if winning_tile == t2 {
                    WaitType::Closed
                } else if winning_tile == t1 {
                    if helpers::tile_to_index(&t1) % 9 == 0 {
                        WaitType::Edge
                    } else {
                        WaitType::TwoSided
                    }
                } else if winning_tile == t3 {
                    if helpers::tile_to_index(&t3) % 9 == 8 {
                        WaitType::Edge
                    } else {
                        WaitType::TwoSided
                    }
                } else {
                    unreachable!("Winning tile in sequence but not t1, t2, or t3");
                }
            }
        }
    }
}


// === Public Function ===

pub fn organize_raw_hand(mut raw_hand: RawHandInput) -> OrganizedHand {
    let mut final_melds = raw_hand.open_melds.clone();
    
    // 1. Create tile counts from ALL 14 tiles
    let mut counts = [0u8; 34];
    for tile in &raw_hand.tiles {
        counts[helpers::tile_to_index(tile)] += 1;
    }

    // 2. Subtract tiles from open melds.
    for meld in &raw_hand.open_melds {
        match meld.meld_type {
            MeldType::Sequence | MeldType::Triplet => {
                counts[helpers::tile_to_index(&meld.tiles[0])] -= 1;
                counts[helpers::tile_to_index(&meld.tiles[1])] -= 1;
                counts[helpers::tile_to_index(&meld.tiles[2])] -= 1;
            }
            MeldType::Kan => {
                counts[helpers::tile_to_index(&meld.tiles[0])] -= 1;
                counts[helpers::tile_to_index(&meld.tiles[1])] -= 1;
                counts[helpers::tile_to_index(&meld.tiles[2])] -= 1;
                counts[helpers::tile_to_index(&meld.tiles[3])] -= 1;
            }
        }
    }

    // 3. Update GameConditions based on open melds
    raw_hand.game_conditions.is_closed_hand = final_melds.is_empty();

    // 4. Determine how many melds we still need to find
    let melds_needed = 4 - final_melds.len();
    
    // --- Case A: 4 open melds (e.g., Hadaka Tanki) ---
    if melds_needed == 0 {
        for i in 0..34 {
            if counts[i] == 2 {
                let pair_tile = helpers::index_to_tile(i);
                let pair = (pair_tile, pair_tile);
                let melds_array = [final_melds[0], final_melds[1], final_melds[2], final_melds[3]];
                let winning_hand = WinningHand {
                    melds: melds_array,
                    pair,
                    winning_tile: raw_hand.winning_tile,
                    wait_type: WaitType::Pair, // Must be a pair wait
                };

                return OrganizedHand {
                    hand_structure: HandStructure::Standard(winning_hand), // <-- Use new enum
                    game_conditions: raw_hand.game_conditions,
                };
            }
        }
        panic!("Invalid hand: 4 open melds but no pair found.");
    }

    // --- Case B: 0-3 open melds ---
    // Try to find a 4-meld, 1-pair hand first
    for i in 0..34 {
        if counts[i] >= 2 {
            // Assume this tile `i` is the pair
            let mut temp_counts = counts;
            temp_counts[i] -= 2;
            let pair = (helpers::index_to_tile(i), helpers::index_to_tile(i));
            let mut closed_melds: Vec<Meld> = Vec::with_capacity(melds_needed);

            // 3. Try to find the remaining melds
            if recursive_parser::find_melds_recursive(&mut temp_counts, &mut closed_melds) {
                if closed_melds.len() == melds_needed {
                    // Success!
                    final_melds.append(&mut closed_melds);
                    let melds_array = [final_melds[0], final_melds[1], final_melds[2], final_melds[3]];

                    let wait_type = wait_analyzer::determine_wait_type(
                        &melds_array,
                        pair,
                        raw_hand.winning_tile
                    );

                    let winning_hand = WinningHand {
                        melds: melds_array,
                        pair,
                        winning_tile: raw_hand.winning_tile,
                        wait_type,
                    };
                    
                    return OrganizedHand {
                        hand_structure: HandStructure::Standard(winning_hand), // <-- Use new enum
                        game_conditions: raw_hand.game_conditions,
                    };
                }
            }
        }
    }

    // --- FAILURE ---
    // If we are here, the 4-meld-1-pair parse failed.
    
    // Now, check for special hands, but ONLY if the hand was closed.
    if raw_hand.game_conditions.is_closed_hand {
        // We must use the tile counts *before* subtracting open melds.
        // Re-create the counts from the original 14 tiles.
        let mut original_counts = [0u8; 34];
        for tile in &raw_hand.tiles {
            original_counts[helpers::tile_to_index(tile)] += 1;
        }

        if let Some(special_hand_structure) = 
            special_yaku_checker::check_special_hands(&raw_hand, &original_counts) 
        {
            return OrganizedHand {
                hand_structure: special_hand_structure,
                game_conditions: raw_hand.game_conditions,
            };
        }
    }

    // If all checks fail, it's an invalid hand.
    panic!("Could not organize hand. Not a valid 4-meld, 1-pair, or special hand.");
}