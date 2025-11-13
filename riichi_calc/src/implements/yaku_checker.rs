//! # Riichi Mahjong Yaku Checker
//!
//! This module takes an organized hand and game state and identifies all
//! winning yaku, including Yakuman and Dora.
//!
//! The main entry point is `check_all_yaku`.

use super::types::{
    game::{AgariType, GameContext, PlayerContext},
    hand::{AgariHand, HandOrganization, HandStructure, Machi, Mentsu, MentsuType},
    // --- REFACTOR: `is_...` helpers removed from imports ---
    tiles::{index_to_tile, tile_to_index, Hai, Jihai, Kaze, Sangenpai, Suhai},
    yaku::Yaku,
};
use std::collections::{HashMap, HashSet};

/// # YakuResult
/// The definitive result from the yaku checker, passed to the score calculator.
#[derive(Debug, Clone)]
pub struct YakuResult {
    /// The confirmed, valid structure of the winning hand.
    pub hand_structure: HandStructure,
    /// A list of all yaku achieved, including Dora.
    pub yaku_list: Vec<Yaku>,
    /// --- NEW: Pass akadora count to scorer ---
    pub num_akadora: u8,
}

// --- Main Public Function ---

/// Checks a hand for all yaku.
pub fn check_all_yaku(
    organization: HandOrganization,
    player: &PlayerContext,
    game: &GameContext,
    agari_type: AgariType,
) -> Result<YakuResult, &'static str> {
    // 1. Check for game-state Yakuman first (Tenhou, etc.)
    let mut yakuman_list = check_game_state_yakuman(player, game);

    // 2. Resolve the hand structure and check for hand-based Yakuman.
    let (hand_structure, hand_yakuman) =
        match resolve_hand_structure(organization, player, game, agari_type) {
            Ok((structure, yakuman)) => (structure, yakuman),
            Err(e) => return Err(e),
        };

    yakuman_list.extend(hand_yakuman);

    // 3. If we have any Yakuman, we are done.
    if !yakuman_list.is_empty() {
        // Post-process to handle double yakuman (e.g., SuuankouTanki replaces Suuankou)
        let final_yakuman = post_process_yakuman(yakuman_list);

        return Ok(YakuResult {
            hand_structure,
            yaku_list: final_yakuman,
            num_akadora: 0, // Akadora don't count for Yakuman
        });
    }

    // 4. No Yakuman. Find regular yaku based on the hand structure.
    let mut regular_yaku: Vec<Yaku> = match &hand_structure {
        HandStructure::YonmentsuIchiatama(agari_hand) => {
            find_standard_yaku(agari_hand, player, game, agari_type)
        }
        HandStructure::Chiitoitsu {
            pairs,
            agari_hai,
            machi,
        } => find_chiitoitsu_yaku(pairs, agari_hai, machi, player, game, agari_type),
        // Kokushi/Chuuren are Yakuman, so they would have returned above.
        _ => vec![],
    };

    // 5. Check for Dora.
    // A hand is only valid if it has at least one yaku OR is in Riichi.
    let has_yaku =
        !regular_yaku.is_empty() || player.is_riichi || player.is_daburu_riichi;

    let mut num_akadora_to_add = 0;

    if has_yaku {
        // Find all tiles in the hand
        let all_tiles = get_all_tiles_from_structure(&hand_structure);

        // Add Dora
        let dora_count = count_dora(&all_tiles, &game.dora_indicators);
        for _ in 0..dora_count {
            regular_yaku.push(Yaku::Dora);
        }

        // Add UraDora (only if Riichi)
        if (player.is_riichi || player.is_daburu_riichi) && !game.uradora_indicators.is_empty() {
            let uradora_count = count_dora(&all_tiles, &game.uradora_indicators);
            for _ in 0..uradora_count {
                regular_yaku.push(Yaku::UraDora);
            }
        }

        // --- NEW: Add AkaDora (Red Fives) ---
        // The `num_akadora` is provided in the `GameContext`.
        // We trust the `raw_hand_organizer`'s validator to have
        // confirmed this number is valid.
        if game.num_akadora > 0 {
            num_akadora_to_add = game.num_akadora;
            for _ in 0..game.num_akadora {
                regular_yaku.push(Yaku::AkaDora);
            }
        }
    }

    Ok(YakuResult {
        hand_structure,
        yaku_list: regular_yaku,
        num_akadora: num_akadora_to_add,
    })
}

// --- 1. Yakuman Checkers ---

/// Checks for Tenhou (Blessing of Heaven) and Chiihou (Blessing of Earth)
fn check_game_state_yakuman(player: &PlayerContext, game: &GameContext) -> Vec<Yaku> {
    let mut yaku = Vec::new();
    if game.is_tenhou {
        yaku.push(Yaku::Tenhou);
    }
    if game.is_chiihou {
        yaku.push(Yaku::Chiihou);
    }
    if game.is_renhou {
        yaku.push(Yaku::Renhou);
    }
    yaku
}

/// Tries to resolve the hand into a structure and find any hand-based Yakuman.
fn resolve_hand_structure(
    org: HandOrganization,
    player: &PlayerContext,
    game: &GameContext,
    agari_type: AgariType,
) -> Result<(HandStructure, Vec<Yaku>), &'static str> {
    match org {
        HandOrganization::YonmentsuIchiatama(agari_hand) => {
            // This is a standard hand. Check for standard-pattern yakuman.
            let (yakuman_list, chuuren_flag) =
                check_standard_yakuman(&agari_hand, player, game, agari_type);

            let structure = if let Some(is_junsei) = chuuren_flag {
                HandStructure::ChuurenPoutou {
                    hand: agari_hand,
                    is_junsei,
                }
            } else {
                HandStructure::YonmentsuIchiatama(agari_hand)
            };

            Ok((structure, yakuman_list))
        }
        HandOrganization::Irregular { counts, agari_hai } => {
            // Try Kokushi first
            if let Some((kokushi_structure, kokushi_yaku)) = check_kokushi(&counts, agari_hai) {
                Ok((kokushi_structure, vec![kokushi_yaku]))
            }
            // Try Chiitoitsu next
            else if let Some(chiitoitsu_structure) = check_chiitoitsu(&counts, agari_hai) {
                // Chiitoitsu can also be Tsuuiisou (All Honors)
                let yakuman = check_chiitoitsu_yakuman(&chiitoitsu_structure);
                Ok((chiitoitsu_structure, yakuman))
            } else {
                Err("Invalid irregular hand. Not Kokushi or Chiitoitsu.")
            }
        }
    }
}

/// Checks a 4-meld, 1-pair hand for all Yakuman.
/// Returns (Yakuman List, ChuurenPoutou flag)
fn check_standard_yakuman(
    hand: &AgariHand,
    player: &PlayerContext,
    _game: &GameContext, // _game is not used, but kept for API consistency
    agari_type: AgariType,
) -> (Vec<Yaku>, Option<bool>) {
    let mut yakuman = Vec::new();
    let all_tiles = get_all_tiles(hand);

    // --- Tile-based Yakuman (Tsuuiisou, Chinroutou, Ryuuiisou) ---
    let mut is_tsuuiisou = true;
    let mut is_chinroutou = true;
    let mut is_ryuuiisou = true;

    for tile in &all_tiles {
        // --- REFACTOR: Use method on Hai ---
        if !tile.is_jihai() {
            is_tsuuiisou = false;
        }
        if !tile.is_terminal() {
            is_chinroutou = false;
        }
        if !is_green_tile(tile) {
            is_ryuuiisou = false;
        }
    }

    if is_tsuuiisou {
        yakuman.push(Yaku::Tsuuiisou);
    }
    if is_chinroutou {
        // Chinroutou requires all tiles to be terminals.
        // This implies it's also Toitoi, but it's a yakuman.
        yakuman.push(Yaku::Chinroutou);
    }
    if is_ryuuiisou {
        // All Green: Only Sou 2,3,4,6,8 and Hatsu.
        yakuman.push(Yaku::Ryuuiisou);
    }

    // --- Meld-based Yakuman ---
    let (koutsu, kantsu) = count_koutsu_kantsu(hand);
    let concealed_koutsu = count_concealed_koutsu(hand, agari_type);

    // Suukantsu (Four Quads)
    if kantsu == 4 {
        yakuman.push(Yaku::Suukantsu);
    }

    // Suuankou (Four Concealed Triplets)
    if concealed_koutsu == 4 {
        if hand.machi == Machi::Tanki {
            yakuman.push(Yaku::SuuankouTanki);
        } else {
            yakuman.push(Yaku::Suuankou);
        }
    }

    // Daisangen (Big Three Dragons)
    let mut dragon_koutsu = 0;
    for mentsu in &hand.mentsu {
        if is_koutsu_or_kantsu(mentsu) {
            if let Hai::Jihai(Jihai::Sangen(_)) = mentsu.tiles[0] {
                dragon_koutsu += 1;
            }
        }
    }
    if dragon_koutsu == 3 {
        yakuman.push(Yaku::Daisangen);
    }

    // Daisuushi / Shousuushi (Big/Little Four Winds)
    let mut wind_koutsu = 0;
    let mut wind_atama = false;
    for mentsu in &hand.mentsu {
        if is_koutsu_or_kantsu(mentsu) {
            if let Hai::Jihai(Jihai::Kaze(_)) = mentsu.tiles[0] {
                wind_koutsu += 1;
            }
        }
    }
    if let Hai::Jihai(Jihai::Kaze(_)) = hand.atama.0 {
        wind_atama = true;
    }

    if wind_koutsu == 4 {
        yakuman.push(Yaku::Daisuushi);
    } else if wind_koutsu == 3 && wind_atama {
        yakuman.push(Yaku::Shousuushi);
    }

    // Chuuren Poutou (Nine Gates)
    let chuuren_flag = check_chuuren(hand);
    if let Some(is_junsei) = chuuren_flag {
        if is_junsei {
            yakuman.push(Yaku::JunseiChuurenPoutou);
        } else {
            yakuman.push(Yaku::ChuurenPoutou);
        }
    }

    (yakuman, chuuren_flag)
}

/// Checks for Kokushi Musou (Thirteen Orphans)
fn check_kokushi(counts: &[u8; 34], agari_hai: Hai) -> Option<(HandStructure, Yaku)> {
    let mut has_pair = false;
    let mut tiles = Vec::new();
    let mut atama_tile = None;

    for (idx, &count) in counts.iter().enumerate() {
        let tile = index_to_tile(idx);
        // --- REFACTOR: Use method on Hai ---
        if !tile.is_yaochuu() {
            if count > 0 {
                return None; // Has a non-yaochuu tile
            }
            continue;
        }

        match count {
            1 => {
                tiles.push(tile);
            }
            2 => {
                if has_pair {
                    return None;
                } // More than one pair
                has_pair = true;
                atama_tile = Some(tile);
                tiles.push(tile); // Add it once
            }
            0 => {
                // This case (count == 0) should be impossible if the hand
                // is valid, as `counts` includes the 14th `agari_hai`.
                // A valid Kokushi will have 13 tiles, 12 of which are
                // 1-of and 1 of which is 2-of.
            }
            _ => return None, // > 2 of a yaocchuu tile
        }
    }

    // We must have a pair
    if !has_pair {
        return None;
    }

    let agari_hai_index = tile_to_index(&agari_hai);
    if counts[agari_hai_index] == 0 {
        return None; // Winning tile not in hand? (Should be impossible)
    }

    let atama = (atama_tile.unwrap(), atama_tile.unwrap());

    // Check for 13-sided wait (Jusanmen)
    // This means the hand *before* agari was 1 of each 13 orphans.
    // So, `counts` (which includes agari_hai) will have one `2` and twelve `1`s.
    // The `agari_hai` *must* be the tile that has a count of 2.
    let mut yaku = Yaku::KokushiMusou;
    let mut final_machi = Machi::KokushiIchimen;

    if atama.0 == agari_hai {
        // This is a 13-sided wait.
        // We already know `agari_hai` is the pair.
        // We just need to confirm all 12 other yaocchuu tiles have a count of 1.
        // `tiles.len()` should be 13 (12 singles + 1 of the pair).
        if tiles.len() == 13 {
            yaku = Yaku::KokushiMusouJusanmen;
            final_machi = Machi::KokushiJusanmen;
        }
    }

    Some((
        HandStructure::KokushiMusou {
            tiles: tiles.try_into().ok()?,
            atama,
            agari_hai,
            machi: final_machi,
        },
        yaku,
    ))
}

/// Checks for Chiitoitsu (Seven Pairs)
fn check_chiitoitsu(counts: &[u8; 34], agari_hai: Hai) -> Option<HandStructure> {
    let mut pair_count = 0;
    let mut pairs = Vec::new();

    for (idx, &count) in counts.iter().enumerate() {
        if count > 0 {
            if count == 2 {
                pair_count += 1;
                let tile = index_to_tile(idx);
                pairs.push((tile, tile));
            } else if count == 4 {
                // 4-of-a-kind counts as 2 pairs in Chiitoitsu
                pair_count += 2;
                let tile = index_to_tile(idx);
                pairs.push((tile, tile));
                pairs.push((tile, tile));
            } else {
                return None; // Has a 1 or 3
            }
        }
    }

    if pair_count == 7 {
        Some(HandStructure::Chiitoitsu {
            pairs: pairs.try_into().ok()?,
            agari_hai,
            machi: Machi::Tanki, // Chiitoitsu is always a pair wait
        })
    } else {
        None
    }
}

/// Checks if a Chiitoitsu is also a Yakuman (Tsuuiisou)
fn check_chiitoitsu_yakuman(hand: &HandStructure) -> Vec<Yaku> {
    if let HandStructure::Chiitoitsu { pairs, .. } = hand {
        let mut is_tsuuiisou = true;
        for (tile, _) in pairs {
            // --- REFACTOR: Use method on Hai ---
            if !tile.is_jihai() {
                is_tsuuiisou = false;
                break;
            }
        }
        if is_tsuuiisou {
            return vec![Yaku::Tsuuiisou];
        }
    }
    vec![]
}

/// Checks for Chuuren Poutou (Nine Gates)
fn check_chuuren(hand: &AgariHand) -> Option<bool> {
    let all_tiles = get_all_tiles(hand);

    // Must be one suit only
    let (is_chinitsu, suit) = check_chinitsu(&all_tiles);
    if !is_chinitsu {
        return None;
    }
    let suit = suit.unwrap(); // We know it's Some

    // Must be menzen
    if !hand.mentsu.iter().all(|m| !m.is_minchou) {
        return None;
    }

    // Check counts: 1,1,1, 2,3,4, 5,6,7, 8,8,8, 9,9,9
    let mut counts = [0u8; 9];
    for tile in &all_tiles {
        if let Hai::Suhai(n, s) = tile {
            if *s == suit {
                counts[(n - 1) as usize] += 1;
            }
        }
    }

    let mut has_extra = false;
    let mut extra_tile_num = 0;

    for (i, &count) in counts.iter().enumerate() {
        let num = i + 1;
        let required_count = if num == 1 || num == 9 { 3 } else { 1 };

        if count < required_count {
            return None;
        }
        if count == required_count + 1 {
            if has_extra {
                return None;
            } // Two extras
            has_extra = true;
            extra_tile_num = num;
        } else if count > required_count + 1 {
            return None; // e.g., five 1s
        }
    }

    if !has_extra {
        return None;
    } // Needs 14 tiles

    // It's Chuuren. Is it Junsei (True 9-sided wait)?
    // This means the winning tile completes the 1,1,1,2,3,4,5,6,7,8,9,9,9 form.
    // The `extra_tile_num` is the number of the tile we have two of.
    // The `agari_hai` must be that number.
    if let Hai::Suhai(n, s) = hand.agari_hai {
        if s == suit && n as usize == extra_tile_num {
            return Some(true); // Junsei!
        }
    }

    Some(false) // Not Junsei, but still Chuuren
}

/// Handles Double Yakuman overrides
fn post_process_yakuman(mut yakuman: Vec<Yaku>) -> Vec<Yaku> {
    let has_suuankou_tanki = yakuman.contains(&Yaku::SuuankouTanki);
    let has_kokushi_jusanmen = yakuman.contains(&Yaku::KokushiMusouJusanmen);
    let has_junsei_chuuren = yakuman.contains(&Yaku::JunseiChuurenPoutou);

    yakuman.retain(|&y| {
        (y != Yaku::Suuankou || !has_suuankou_tanki)
            && (y != Yaku::KokushiMusou || !has_kokushi_jusanmen)
            && (y != Yaku::ChuurenPoutou || !has_junsei_chuuren)
    });

    yakuman
}

// --- 2. Regular Yaku Checkers ---

/// Finds all standard yaku for a 4-meld, 1-pair hand.
fn find_standard_yaku(
    hand: &AgariHand,
    player: &PlayerContext,
    game: &GameContext,
    agari_type: AgariType,
) -> Vec<Yaku> {
    let mut yaku_list = Vec::new();

    // --- State-based Yaku (1 han) ---
    if player.is_daburu_riichi {
        yaku_list.push(Yaku::DaburuRiichi);
    } else if player.is_riichi {
        yaku_list.push(Yaku::Riichi);
    }
    if player.is_ippatsu {
        yaku_list.push(Yaku::Ippatsu);
    }
    if player.is_menzen && agari_type == AgariType::Tsumo {
        yaku_list.push(Yaku::MenzenTsumo);
    }
    if game.is_haitei && agari_type == AgariType::Tsumo {
        yaku_list.push(Yaku::HaiteiRaoyue);
    }
    if game.is_houtei && agari_type == AgariType::Ron {
        yaku_list.push(Yaku::HouteiRaoyui);
    }
    if game.is_rinshan {
        yaku_list.push(Yaku::RinshanKaihou);
    }
    if game.is_chankan {
        yaku_list.push(Yaku::Chankan);
    }

    // --- Yakuhai (1 han) ---
    yaku_list.extend(check_yakuhai(hand, player, game));

    // --- Pinfu (1 han) ---
    if check_pinfu(hand, player, game) {
        yaku_list.push(Yaku::Pinfu);
    }

    // --- Tanyao (1 han) ---
    if check_tanyao(hand) {
        yaku_list.push(Yaku::Tanyao);
    }

    // --- Sequence Yaku (Iipeikou, Ryanpeikou, Sanshoku, Ittsu) ---
    let shuntsu: Vec<&Mentsu> = hand
        .mentsu
        .iter()
        .filter(|m| m.mentsu_type == MentsuType::Shuntsu)
        .collect();

    if player.is_menzen {
        let (iipeikou, ryanpeikou) = check_peikou(&shuntsu);
        if ryanpeikou {
            yaku_list.push(Yaku::Ryanpeikou);
        } else if iipeikou {
            yaku_list.push(Yaku::Iipeikou);
        }
    }

    if check_sanshoku_doujun(&shuntsu) {
        yaku_list.push(Yaku::SanshokuDoujun);
    }

    if check_ittsu(&shuntsu) {
        yaku_list.push(Yaku::Ittsu);
    }

    // --- Triplet Yaku (Toitoi, Sanankou, Sanshoku, Sankantsu, Shousangen) ---
    let (koutsu, kantsu) = count_koutsu_kantsu(hand);

    if koutsu + kantsu == 4 {
        yaku_list.push(Yaku::Toitoi);
    } else {
        // Sanankou and Toitoi are mutually exclusive
        let concealed_koutsu = count_concealed_koutsu(hand, agari_type);
        if concealed_koutsu == 3 {
            yaku_list.push(Yaku::Sanankou);
        }
    }

    if kantsu == 3 {
        yaku_list.push(Yaku::Sankantsu);
    }

    if check_sanshoku_doukou(hand) {
        yaku_list.push(Yaku::SanshokuDoukou);
    }

    if check_shousangen(hand) {
        yaku_list.push(Yaku::Shousangen);
    }

    // --- Terminal/Honor Yaku (Chanta, Junchan, Honroutou) ---
    let all_tiles = get_all_tiles(hand);
    let all_groups = get_all_groups(hand);

    // --- REFACTOR: Use method on Hai ---
    let is_honroutou = all_tiles.iter().all(|t| t.is_yaochuu())
        && !all_tiles.iter().all(|t| t.is_terminal()); // Exclude Chinroutou

    if is_honroutou {
        yaku_list.push(Yaku::Honroutou);
    } else {
        let (is_chanta, is_junchan) = check_chanta_junchan(&all_groups);
        if is_junchan {
            yaku_list.push(Yaku::Junchan);
        } else if is_chanta {
            yaku_list.push(Yaku::Chanta);
        }
    }

    // --- Color Yaku (Honitsu, Chinitsu) ---
    let (is_chinitsu, _) = check_chinitsu(&all_tiles);
    if is_chinitsu {
        yaku_list.push(Yaku::Chinitsu);
    } else {
        let (is_honitsu, _) = check_honitsu(&all_tiles);
        if is_honitsu {
            yaku_list.push(Yaku::Honitsu);
        }
    }

    // --- Yaku Overlap Rules ---
    // Pinfu is incompatible with yaku that give fu.
    if yaku_list.contains(&Yaku::Pinfu) {
        if yaku_list.contains(&Yaku::RinshanKaihou)
            || yaku_list.contains(&Yaku::Chankan)
        // Pinfu and MenzenTsumo *can* stack.
        // Pinfu implies menzen, so no need to check other open-hand yaku.
        // Pinfu is also incompatible with koutsu/kantsu, so
        // Toitoi, Sanankou, Sankantsu, Shousangen, etc. are impossible.
        {
            yaku_list.retain(|&y| y != Yaku::Pinfu);
        }
    }

    yaku_list
}

/// Finds all yaku for a Chiitoitsu hand.
fn find_chiitoitsu_yaku(
    pairs: &[(Hai, Hai); 7],
    _agari_hai: &Hai,
    _machi: &Machi,
    player: &PlayerContext,
    game: &GameContext,
    agari_type: AgariType,
) -> Vec<Yaku> {
    let mut yaku_list = Vec::new();

    // Chiitoitsu is always 2 han and menzen
    yaku_list.push(Yaku::Chiitoitsu);

    // --- State-based Yaku ---
    // Riichi/DaburuRiichi/Ippatsu
    if player.is_daburu_riichi {
        yaku_list.push(Yaku::DaburuRiichi);
    } else if player.is_riichi {
        yaku_list.push(Yaku::Riichi);
    }
    if player.is_ippatsu {
        yaku_list.push(Yaku::Ippatsu);
    }
    // MenzenTsumo
    if agari_type == AgariType::Tsumo {
        yaku_list.push(Yaku::MenzenTsumo);
    }
    // Haitei/Houtei
    if game.is_haitei && agari_type == AgariType::Tsumo {
        yaku_list.push(Yaku::HaiteiRaoyue);
    }
    if game.is_houtei && agari_type == AgariType::Ron {
        yaku_list.push(Yaku::HouteiRaoyui);
    }

    // --- Tile-based Yaku ---
    let all_tiles: Vec<Hai> = pairs.iter().flat_map(|&(t1, t2)| vec![t1, t2]).collect();

    // --- REFACTOR: Use method on Hai ---
    if all_tiles.iter().all(|t| t.is_simple()) {
        yaku_list.push(Yaku::Tanyao);
    }

    // Honroutou (All Terminals & Honors)
    if all_tiles.iter().all(|t| t.is_yaochuu()) {
        yaku_list.push(Yaku::Honroutou);
    }

    // Color Yaku (Honitsu, Chinitsu)
    let (is_chinitsu, _) = check_chinitsu(&all_tiles);
    if is_chinitsu {
        yaku_list.push(Yaku::Chinitsu);
    } else {
        let (is_honitsu, _) = check_honitsu(&all_tiles);
        if is_honitsu {
            yaku_list.push(Yaku::Honitsu);
        }
    }

    yaku_list
}

// --- 3. Dora Checkers ---

/// Counts the number of Dora in a hand.
fn count_dora(all_tiles: &[Hai], indicators: &[Hai]) -> u8 {
    let mut count = 0;
    for indicator in indicators {
        let dora_tile = get_dora_tile(indicator);
        for tile in all_tiles {
            if *tile == dora_tile {
                count += 1;
            }
        }
    }
    count
}

/// Gets the Dora tile from an indicator.
fn get_dora_tile(indicator: &Hai) -> Hai {
    match indicator {
        Hai::Suhai(n, s) => {
            if *n == 9 {
                Hai::Suhai(1, *s)
            } else {
                Hai::Suhai(n + 1, *s)
            }
        }
        Hai::Jihai(Jihai::Kaze(k)) => Hai::Jihai(Jihai::Kaze(match k {
            Kaze::Ton => Kaze::Nan,
            Kaze::Nan => Kaze::Shaa,
            Kaze::Shaa => Kaze::Pei,
            Kaze::Pei => Kaze::Ton,
        })),
        Hai::Jihai(Jihai::Sangen(s)) => Hai::Jihai(Jihai::Sangen(match s {
            Sangenpai::Haku => Sangenpai::Hatsu,
            Sangenpai::Hatsu => Sangenpai::Chun,
            Sangenpai::Chun => Sangenpai::Haku,
        })),
    }
}

// --- 4. Yaku-specific Helper Functions ---

// --- REFACTOR: Corrected and simplified check_yakuhai ---
/// Checks for Yakuhai (Dragons, Seat Wind, Prevalent Wind).
fn check_yakuhai(
    hand: &AgariHand,
    player: &PlayerContext,
    game: &GameContext,
) -> Vec<Yaku> {
    let mut yaku = Vec::new();

    // Collect all koutsu/kantsu tiles
    let koutsu_tiles: HashSet<Hai> = hand
        .mentsu
        .iter()
        .filter(|m| is_koutsu_or_kantsu(m))
        .map(|m| m.tiles[0])
        .collect();

    // Check Dragons
    if koutsu_tiles.contains(&Hai::Jihai(Jihai::Sangen(Sangenpai::Haku))) {
        yaku.push(Yaku::YakuhaiSangenpai);
    }
    if koutsu_tiles.contains(&Hai::Jihai(Jihai::Sangen(Sangenpai::Hatsu))) {
        yaku.push(Yaku::YakuhaiSangenpai);
    }
    if koutsu_tiles.contains(&Hai::Jihai(Jihai::Sangen(Sangenpai::Chun))) {
        yaku.push(Yaku::YakuhaiSangenpai);
    }

    // Check Winds
    let bakaze_hai = Hai::Jihai(Jihai::Kaze(game.bakaze));
    if koutsu_tiles.contains(&bakaze_hai) {
        yaku.push(Yaku::YakuhaiBakaze);
    }

    let jikaze_hai = Hai::Jihai(Jihai::Kaze(player.jikaze));
    // Avoid double-counting if jikaze == bakaze
    if koutsu_tiles.contains(&jikaze_hai) && jikaze_hai != bakaze_hai {
        yaku.push(Yaku::YakuhaiJikaze);
    } else if koutsu_tiles.contains(&jikaze_hai) && jikaze_hai == bakaze_hai {
        // We already added YakuhaiBakaze. Now we add Jikaze.
        // This is correct, as it counts for 2 han.
        yaku.push(Yaku::YakuhaiJikaze);
    }

    yaku
}

/// Checks for Pinfu (No-points hand).
fn check_pinfu(hand: &AgariHand, player: &PlayerContext, game: &GameContext) -> bool {
    // 1. Must be menzen
    if !player.is_menzen {
        return false;
    }
    // 2. All 4 melds are Shuntsu
    if !hand
        .mentsu
        .iter()
        .all(|m| m.mentsu_type == MentsuType::Shuntsu)
    {
        return false;
    }
    // 3. Atama is not a Yakuhai tile
    if let Hai::Jihai(Jihai::Sangen(_)) = hand.atama.0 {
        return false;
    }
    if let Hai::Jihai(Jihai::Kaze(k)) = hand.atama.0 {
        if k == game.bakaze || k == player.jikaze {
            return false;
        }
    }
    // 4. Must be a Ryanmen (two-sided) wait
    if hand.machi != Machi::Ryanmen {
        return false;
    }

    true
}

/// Checks for Tanyao (All Simples).
fn check_tanyao(hand: &AgariHand) -> bool {
    // --- REFACTOR: Use method on Hai ---
    get_all_tiles(hand).iter().all(|t| t.is_simple())
}

/// Checks for Iipeikou (Pure Double Sequence) and Ryanpeikou (Twice Pure Double Sequence)
fn check_peikou<'a>(shuntsu: &[&'a Mentsu]) -> (bool, bool) {
    if shuntsu.len() < 2 {
        return (false, false);
    }

    let mut identical_pairs = 0;
    let mut seen = HashSet::new();

    for (i, m1) in shuntsu.iter().enumerate() {
        if seen.contains(&i) {
            continue;
        }
        for (j, m2) in shuntsu.iter().enumerate() {
            if i == j || seen.contains(&j) {
                continue;
            }

            // Check for identical shuntsu (e.g., 234m and 234m)
            if m1.tiles[0] == m2.tiles[0] {
                identical_pairs += 1;
                seen.insert(i);
                seen.insert(j);
                break;
            }
        }
    }

    (identical_pairs == 1, identical_pairs == 2)
}

/// Checks for Sanshoku Doujun (Mixed Triple Sequence)
fn check_sanshoku_doujun<'a>(shuntsu: &[&'a Mentsu]) -> bool {
    if shuntsu.len() < 3 {
        return false;
    }

    let mut starters: HashMap<u8, (bool, bool, bool)> = HashMap::new();

    for m in shuntsu {
        if let Hai::Suhai(n, s) = m.tiles[0] {
            let entry = starters.entry(n).or_insert((false, false, false));
            match s {
                Suhai::Manzu => entry.0 = true,
                Suhai::Pinzu => entry.1 = true,
                Suhai::Souzu => entry.2 = true,
            }
        }
    }

    starters.values().any(|&(m, p, s)| m && p && s)
}

/// Checks for Ittsu (Pure Straight)
fn check_ittsu<'a>(shuntsu: &[&'a Mentsu]) -> bool {
    if shuntsu.len() < 3 {
        return false;
    }

    let mut suits: HashMap<Suhai, HashSet<u8>> = HashMap::new();

    for m in shuntsu {
        if let Hai::Suhai(n, s) = m.tiles[0] {
            suits.entry(s).or_default().insert(n);
        }
    }

    for set in suits.values() {
        if set.contains(&1) && set.contains(&4) && set.contains(&7) {
            return true;
        }
    }
    false
}

/// Checks for Sanshoku Doukou (Triple Triplets)
fn check_sanshoku_doukou(hand: &AgariHand) -> bool {
    let koutsu: Vec<&Mentsu> = hand
        .mentsu
        .iter()
        .filter(|m| is_koutsu_or_kantsu(m))
        .collect();

    if koutsu.len() < 3 {
        return false;
    }

    let mut numbers: HashMap<u8, (bool, bool, bool)> = HashMap::new();

    for m in koutsu {
        if let Hai::Suhai(n, s) = m.tiles[0] {
            let entry = numbers.entry(n).or_insert((false, false, false));
            match s {
                Suhai::Manzu => entry.0 = true,
                Suhai::Pinzu => entry.1 = true,
                Suhai::Souzu => entry.2 = true,
            }
        }
    }

    numbers.values().any(|&(m, p, s)| m && p && s)
}

/// Checks for Shousangen (Little Three Dragons)
fn check_shousangen(hand: &AgariHand) -> bool {
    let mut dragon_koutsu = 0;
    let mut dragon_atama = false;

    for m in &hand.mentsu {
        if is_koutsu_or_kantsu(m) {
            if let Hai::Jihai(Jihai::Sangen(_)) = m.tiles[0] {
                dragon_koutsu += 1;
            }
        }
    }

    if let Hai::Jihai(Jihai::Sangen(_)) = hand.atama.0 {
        dragon_atama = true;
    }

    dragon_koutsu == 2 && dragon_atama
}

/// Checks for Chanta and Junchan
fn check_chanta_junchan(groups: &[Vec<Hai>]) -> (bool, bool) {
    let mut is_chanta = true;
    let mut is_junchan = true; // Assumed true until a Jihai is found

    for group in groups {
        let mut has_terminal = false;
        let mut has_jihai = false;

        for tile in group {
            // --- REFACTOR: Use method on Hai ---
            if tile.is_jihai() {
                has_jihai = true;
            }
            if tile.is_terminal() {
                has_terminal = true;
            }
        }

        if !has_terminal && !has_jihai {
            is_chanta = false;
            is_junchan = false;
            break;
        }

        if has_jihai {
            is_junchan = false; // Has an honor, so can't be Junchan
        }
    }

    (is_chanta, is_junchan && is_chanta)
}

/// Checks for Honitsu (Half Flush) and Chinitsu (Full Flush)
fn check_color(all_tiles: &[Hai]) -> (bool, bool, Option<Suhai>) {
    let mut suit = None;
    let mut has_jihai = false;
    let mut is_honitsu = true;
    let mut is_chinitsu = true;

    for tile in all_tiles {
        match tile {
            Hai::Suhai(_n, s) => {
                if suit.is_none() {
                    suit = Some(*s);
                } else if suit != Some(*s) {
                    is_honitsu = false;
                    is_chinitsu = false;
                    break;
                }
            }
            Hai::Jihai(_) => {
                has_jihai = true;
                is_chinitsu = false;
            }
        }
    }

    if !has_jihai && suit.is_none() {
        // All Jihai (Tsuuiisou)
        is_honitsu = false;
        is_chinitsu = false;
    }

    if !has_jihai {
        is_honitsu = false;
    } // Chinitsu is not Honitsu

    (is_honitsu, is_chinitsu, suit)
}

fn check_honitsu(all_tiles: &[Hai]) -> (bool, Option<Suhai>) {
    let (hon, _chin, suit) = check_color(all_tiles);
    (hon, suit)
}
fn check_chinitsu(all_tiles: &[Hai]) -> (bool, Option<Suhai>) {
    let (_hon, chin, suit) = check_color(all_tiles);
    (chin, suit)
}

// --- 5. Generic Tile Helpers ---

/// Gets all 14 tiles from a standard hand.
fn get_all_tiles(hand: &AgariHand) -> Vec<Hai> {
    let mut tiles = Vec::with_capacity(14);
    tiles.push(hand.atama.0);
    tiles.push(hand.atama.1);
    for mentsu in &hand.mentsu {
        match mentsu.mentsu_type {
            MentsuType::Shuntsu | MentsuType::Koutsu => {
                tiles.extend_from_slice(&mentsu.tiles[0..3]);
            }
            MentsuType::Kantsu => {
                tiles.extend_from_slice(&mentsu.tiles[0..4]);
            }
        }
    }
    tiles
}

/// Gets all 14 tiles from any hand structure.
fn get_all_tiles_from_structure(structure: &HandStructure) -> Vec<Hai> {
    match structure {
        HandStructure::YonmentsuIchiatama(hand) => get_all_tiles(hand),
        HandStructure::Chiitoitsu { pairs, .. } => {
            pairs.iter().flat_map(|&(t1, t2)| vec![t1, t2]).collect()
        }
        HandStructure::KokushiMusou { tiles, atama, .. } => {
            let mut v = tiles.to_vec();
            v.push(atama.0); // The pair tile
            v
        }
        HandStructure::ChuurenPoutou { hand, .. } => get_all_tiles(hand),
    }
}

/// Gets all 5 "groups" (4 melds + 1 pair) from a hand.
fn get_all_groups(hand: &AgariHand) -> Vec<Vec<Hai>> {
    let mut groups = Vec::with_capacity(5);
    groups.push(vec![hand.atama.0, hand.atama.1]);
    for mentsu in &hand.mentsu {
        match mentsu.mentsu_type {
            MentsuType::Shuntsu | MentsuType::Koutsu => {
                groups.push(mentsu.tiles[0..3].to_vec());
            }
            MentsuType::Kantsu => {
                groups.push(mentsu.tiles[0..4].to_vec());
            }
        }
    }
    groups
}

/// Counts koutsu and kantsu.
fn count_koutsu_kantsu(hand: &AgariHand) -> (u8, u8) {
    let mut koutsu = 0;
    let mut kantsu = 0;
    for m in &hand.mentsu {
        match m.mentsu_type {
            MentsuType::Koutsu => koutsu += 1,
            MentsuType::Kantsu => kantsu += 1,
            _ => (),
        }
    }
    (koutsu, kantsu)
}

/// Counts concealed koutsu/kantsu (for Sanankou/Suuankou).
fn count_concealed_koutsu(hand: &AgariHand, agari_type: AgariType) -> u8 {
    let mut count = 0;
    for m in &hand.mentsu {
        if m.is_minchou {
            continue;
        } // Skip open melds

        if m.mentsu_type == MentsuType::Koutsu {
            // If Ron, the koutsu completed by the agari_hai is NOT concealed.
            if agari_type == AgariType::Ron {
                // We only need to check one tile, as all are the same
                if m.tiles[0] == hand.agari_hai {
                    continue; // This triplet was completed by Ron, not concealed.
                }
            }
            count += 1;
        } else if m.mentsu_type == MentsuType::Kantsu {
            count += 1; // Concealed Kantsu (Ankan) always counts
        }
    }
    count
}

/// Is the meld a Koutsu or Kantsu?
fn is_koutsu_or_kantsu(mentsu: &Mentsu) -> bool {
    mentsu.mentsu_type == MentsuType::Koutsu || mentsu.mentsu_type == MentsuType::Kantsu
}

/// Is the tile part of Ryuuiisou (All Green)?
fn is_green_tile(tile: &Hai) -> bool {
    match tile {
        Hai::Suhai(n, Suhai::Souzu) => {
            *n == 2 || *n == 3 || *n == 4 || *n == 6 || *n == 8
        }
        Hai::Jihai(Jihai::Sangen(Sangenpai::Hatsu)) => true,
        _ => false,
    }
}