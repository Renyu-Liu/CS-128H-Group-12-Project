/// # Core Tile Definitions
///
/// This module defines the most basic components of a Mahjong tile.
pub mod tiles {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// Represents the three numbered suits.
    pub enum Suhai {
        // 数牌 (Numbered Tiles)
        Manzu, // 萬子 (Characters)
        Pinzu, // 筒子 (Circles)
        Souzu, // 索子 (Bamboo)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// Represents the four wind directions.
    pub enum Kaze {
        // 風牌 (Wind Tiles)
        Ton,  // 東 (East)
        Nan,  // 南 (South)
        Shaa, // 西 (West)
        Pei,  // 北 (North)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// Represents the three dragons.
    pub enum Sangenpai {
        // 三元牌 (Dragon Tiles)
        Haku, // 白 (White)
        Hatsu, // 發 (Green)
        Chun, // 中 (Red)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// Represents any honor tile (Wind or Dragon).
    pub enum Jihai {
        // 字牌 (Honor Tiles)
        Kaze(Kaze),
        Sangen(Sangenpai),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// Represents a single Mahjong tile.
    pub enum Hai {
        // 牌 (Tile)
        Suhai(u8, Suhai), // 数牌 (Numbered tile, 1-9)
        Jihai(Jihai),     // 字牌 (Honor tile)
    }

    // --- NEW: Tile helper methods ---
    /// This implementation block consolidates all tile-checking logic.
    impl Hai {
        /// Is the tile a simple (2-8 suhai)?
        pub fn is_simple(&self) -> bool {
            match self {
                Hai::Suhai(n, _) => *n >= 2 && *n <= 8,
                Hai::Jihai(_) => false,
            }
        }

        /// Is the tile a terminal (1 or 9)?
        pub fn is_terminal(&self) -> bool {
            match self {
                Hai::Suhai(n, _) => *n == 1 || *n == 9,
                Hai::Jihai(_) => false,
            }
        }

        /// Is the tile an honor tile (wind or dragon)?
        pub fn is_jihai(&self) -> bool {
            matches!(self, Hai::Jihai(_))
        }

        /// Is the tile a terminal or honor (yaochuu-hai)?
        pub fn is_yaochuu(&self) -> bool {
            self.is_terminal() || self.is_jihai()
        }
    }
    // --- End of new block ---

    /// Converts a Hai enum to its corresponding index (0-33).
    pub fn tile_to_index(tile: &Hai) -> usize {
        match tile {
            Hai::Suhai(n, Suhai::Manzu) => (*n - 1) as usize, // 0-8
            Hai::Suhai(n, Suhai::Pinzu) => (*n - 1) as usize + 9, // 9-17
            Hai::Suhai(n, Suhai::Souzu) => (*n - 1) as usize + 18, // 18-26
            Hai::Jihai(Jihai::Kaze(Kaze::Ton)) => 27,
            Hai::Jihai(Jihai::Kaze(Kaze::Nan)) => 28,
            Hai::Jihai(Jihai::Kaze(Kaze::Shaa)) => 29,
            Hai::Jihai(Jihai::Kaze(Kaze::Pei)) => 30,
            Hai::Jihai(Jihai::Sangen(Sangenpai::Haku)) => 31,
            Hai::Jihai(Jihai::Sangen(Sangenpai::Hatsu)) => 32,
            Hai::Jihai(Jihai::Sangen(Sangenpai::Chun)) => 33,
        }
    }

    /// Converts an index (0-33) back into a Hai.
    pub fn index_to_tile(index: usize) -> Hai {
        match index {
            0..=8 => Hai::Suhai((index + 1) as u8, Suhai::Manzu),
            9..=17 => Hai::Suhai(((index - 9) + 1) as u8, Suhai::Pinzu),
            18..=26 => Hai::Suhai(((index - 18) + 1) as u8, Suhai::Souzu),
            27 => Hai::Jihai(Jihai::Kaze(Kaze::Ton)),
            28 => Hai::Jihai(Jihai::Kaze(Kaze::Nan)),
            29 => Hai::Jihai(Jihai::Kaze(Kaze::Shaa)),
            30 => Hai::Jihai(Jihai::Kaze(Kaze::Pei)),
            31 => Hai::Jihai(Jihai::Sangen(Sangenpai::Haku)),
            32 => Hai::Jihai(Jihai::Sangen(Sangenpai::Hatsu)),
            33 => Hai::Jihai(Jihai::Sangen(Sangenpai::Chun)),
            _ => panic!("Invalid tile index: {}", index),
        }
    }
}

/// # Hand Structure and Composition
///
/// This module defines how a hand is constructed, including melds, waits,
/// and the overall valid hand patterns.
pub mod hand {
    use super::tiles::Hai;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Represents the type of a meld (group of tiles).
    pub enum MentsuType {
        /// 面子の種類 (Type of Meld)
        Shuntsu, // 順子 (Sequence)
        Koutsu, // 刻子 (Triplet)
        Kantsu, // 槓子 (Kan/Quad)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Represents a single meld in the hand.
    pub struct Mentsu {
        /// 面子 (Meld)
        pub mentsu_type: MentsuType,
        pub is_minchou: bool, // 明張 (Is the meld open?)
        pub tiles: [Hai; 4], // Use 4 tiles; for Shuntsu/Koutsu, the 4th is unused.
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Represents the type of wait for a winning hand.
    pub enum Machi {
        // 待ちの種類 (Type of Wait)
        Ryanmen, // 両面 (Two-Sided)
        Tanki,   // 単騎 (Pair wait)
        Penchan, // 辺張 (Edge wait, e.g., 1-2 waiting on 3)
        Kanchan, // 嵌張 (Closed wait, e.g., 4-6 waiting on 5)
        Shanpon, // 双碰 (Triplet-pair wait)

        // Special waits for Yakuman
        KokushiIchimen,    // 国士一面 (Kokushi single wait)
        KokushiJusanmen, // 国士十三面 (Kokushi 13-sided wait)
    }

    #[derive(Debug, Clone, Copy)]
    /// Represents a standard 4-meld, 1-pair winning hand.
    pub struct AgariHand {
        // 和了手 (Winning Hand)
        pub mentsu: [Mentsu; 4], // 面子 (The 4 melds)
        pub atama: (Hai, Hai),  // 頭 (The pair)
        pub agari_hai: Hai,     // 和了牌 (The winning tile)
        pub machi: Machi,       // 待ち (The wait type)
    }

    /// Represents the two possible outcomes of the raw hand organizer.
    #[derive(Debug, Clone)]
    pub enum HandOrganization {
        /// Standard 4 melds, 1 pair. Ready for standard yaku checking.
        YonmentsuIchiatama(AgariHand), // 四面子一頭 (Standard Hand: 4 Melds, 1 Pair)
        /// An irregular hand (e.g., Chiitoitsu, Kokushi) or an invalid parse.
        /// The yaku checker will determine which, using the raw counts.
        Irregular {
            // 非標準手 (Irregular Hand)
            /// The raw 14-tile counts of the *entire* hand.
            counts: [u8; 34],
            /// The winning tile, needed for yaku checking.
            agari_hai: Hai,
        },
    }

    #[derive(Debug, Clone)]
    /// Represents the valid, recognized structure of a winning hand.
    /// (This is what the *yaku checker* will ultimately produce)
    pub enum HandStructure {
        /// 四面子一頭 (Standard 4 melds, 1 pair)
        YonmentsuIchiatama(AgariHand),

        /// 七対子 (Seven Pairs)
        Chiitoitsu {
            pairs: [(Hai, Hai); 7],
            agari_hai: Hai,
            machi: Machi, // Will always be Machi::Tanki
        },

        /// 国士無双 (Thirteen Orphans)
        KokushiMusou {
            tiles: [Hai; 13], // The 13 unique tiles
            atama: (Hai, Hai),  // The pair
            // --- FIX: Prefixed `agari_hai` with `_` to silence the dead code warning. ---
            _agari_hai: Hai, // The winning tile
            // --- FIX: Prefixed `machi` with `_` to silence the dead code warning. ---
            _machi: Machi, // KokushiIchimen (1-sided wait) or KokushiJusanmen (13-sided wait)
        },

        /// 九蓮宝燈 (Nine Gates)
        /// This structure is only noted for calculating the Junsei (True) wait.
        /// The hand is still passed as a YonmentsuIchiatama to the scorer,
        /// but this flag is used to award the double yakuman.
        ChuurenPoutou {
            hand: AgariHand,
            // --- FIX: Prefixed `is_junsei` with `_` to silence the dead code warning. ---
            _is_junsei: bool, // 純正 (Is it a true 9-sided wait?)
        },
    }
}

/// # Game State and Context
///
/// This module defines the context of the game when a hand is won,
/// which is necessary for calculating certain yaku (e.g., Yakuhai, Haitei).
pub mod game {
    use super::tiles::{Hai, Kaze};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Represents how the hand was won.
    pub enum AgariType {
        Tsumo, // 自摸 (Self-draw)
        Ron,   // 栄和 (Win off discard)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// Context for the player winning the hand.
    pub struct PlayerContext {
        pub jikaze: Kaze,    // 自風 (Seat Wind)
        pub is_oya: bool,    // 親 (Is player the dealer?)
        pub is_riichi: bool, // 立直 (Was Riichi declared?)
        pub is_daburu_riichi: bool, // ダブル立直 (Double Riichi)
        pub is_ippatsu: bool,         // 一発 (Ippatsu)
        pub is_menzen: bool,        // 門前 (Is the hand fully concealed?)
    }

    #[derive(Debug, Clone)]
    /// Context for the current round of play.
    pub struct GameContext {
        pub bakaze: Kaze,             // 場風 (Prevalent Wind)
        pub kyoku: u8,                // 局 (Round number, e.g., 1 for East 1)
        pub honba: u8,                // 本場 (Honba counter)
        pub riichi_bou: u8,           // リーチ棒 (Riichi sticks on table)
        pub dora_indicators: Vec<Hai>,  // ドラ表示牌 (Dora indicators)
        pub uradora_indicators: Vec<Hai>, // 裏ドラ表示牌 (Ura Dora indicators)
        pub num_akadora: u8,          // 赤ドラの数 (Number of Red Dora in the hand)
        // Special win condition flags
        pub is_tenhou: bool,    // 天和 (Blessing of Heaven)
        pub is_chiihou: bool,   // 地和 (Blessing of Earth)
        pub is_renhou: bool,    // 人和 (Blessing of Man)
        pub is_haitei: bool,    // 海底 (Under the Sea - last draw)
        pub is_houtei: bool,    // 河底 (Under the River - last discard)
        pub is_rinshan: bool,   // 嶺上 (After a Kan)
        pub is_chankan: bool,   // 搶槓 (Robbing a Kan)
    }
}

/// # Yaku (Winning Hands)
///
/// This module defines all possible yaku, including Dora.
pub mod yaku {
    // --- NEW ---
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    /// Represents a single Yaku (or bonus Dora) awarded to a hand.
    /// The han value is determined by the scoring logic, considering
    /// open/closed state.
    pub enum Yaku {
        // 役 (Yaku)
        // --- 1 Han Yaku ---
        Riichi,           // 立直 (Riichi)
        Ippatsu,          // 一発 (Ippatsu)
        MenzenTsumo,      // 門前清自摸和 (Fully Concealed Hand)
        Pinfu,            // 平和 (No-Points Hand)
        Iipeikou,         // 一盃口 (Pure Double Sequence)
        HaiteiRaoyue,     // 海底撈月 (Under the Sea)
        HouteiRaoyui,     // 河底撈魚 (Under the River)
        RinshanKaihou,    // 嶺上開花 (After a Kan)
        Chankan,          // 搶槓 (Robbing a Kan)
        Tanyao,           // 断幺九 (All Simples)
        YakuhaiJikaze,    // 役牌: 自風 (Seat Wind)
        YakuhaiBakaze,    // 役牌: 場風 (Prevalent Wind)
        YakuhaiSangenpai, // 役牌: 三元牌 (Dragon)

        // --- 2 Han Yaku ---
        DaburuRiichi,   // ダブル立直 (Double Riichi)
        Chiitoitsu,     // 七対子 (Seven Pairs)
        SanshokuDoujun, // 三色同順 (Mixed Triple Sequence) kuisagari
        Ittsu,          // 一気通貫 (Pure Straight) kuisagari
        Chanta,         // 全帯幺九 (Half Outside Hand) kuisagari
        Toitoi,         // 対々和 (All Triplets)
        Sanankou,       // 三暗刻 (Three Concealed Triplets)
        SanshokuDoukou, // 三色同刻 (Triple Triplets)
        Sankantsu,      // 三槓子 (Three Quads)
        Shousangen,     // 小三元 (Little Three Dragons)
        Honroutou,      // 混老頭 (All Terminals and Honors)

        // --- 3 Han Yaku ---
        Ryanpeikou, // 二盃口 (Twice Pure Double Sequence)
        Junchan,    // 純全帯么 (Fully Outside Hand) kuisagari
        Honitsu,    // 混一色 (Half Flush) kuisagari

        // --- 6 Han Yaku ---
        Chinitsu, // 清一色 (Full Flush) kuisagari

        // --- Yakuman (13 Han) ---
        Tenhou,               // 天和 (Blessing of Heaven)
        Chiihou,              // 地和 (Blessing of Earth)
        Renhou,               // 人和 (Blessing of Man)
        Daisangen,            // 大三元 (Big Three Dragons)
        Suuankou,             // 四暗刻 (Four Concealed Triplets)
        Daisuushi,            // 大四喜 (Four Big Winds)
        Shousuushi,           // 小四喜 (Four Little Winds)
        Tsuuiisou,            // 字一色 (All Honors)
        Chinroutou,           // 清老頭 (All Terminals)
        Ryuuiisou,            // 緑一色 (All Green)
        Suukantsu,            // 四槓子 (Four Quads)
        KokushiMusou,         // 国士無双 (Thirteen Orphans)
        ChuurenPoutou,        // 九蓮宝燈 (Nine Gates)

        // --- Double Yakuman (26 Han) ---
        SuuankouTanki,        // 四暗刻単騎 (Single Wait Four Concealed)
        KokushiMusouJusanmen, // 国士無S双13面待ち (13-Sided Wait Kokushi)
        JunseiChuurenPoutou,  // 純正九蓮宝燈 (True Nine Gates)

        // --- Dora (Bonus Han, not Yaku) ---
        Dora,    // ドラ (Dora)
        UraDora, // 裏ドラ (Ura Dora)
        AkaDora, // 赤ドラ (Red Five Dora)
    }

    // --- NEW: Implement Display for Yaku ---
    impl fmt::Display for Yaku {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // We use `Debug` format for a simple, readable name.
            write!(f, "{:?}", self)
        }
    }
}

/// # Scoring Results
///
/// This module defines the final output of a score calculation.
pub mod scoring {
    // --- NEW ---
    use std::fmt;
    // --- MODIFIED: Import AgariType for the struct ---
    use super::game::AgariType;
    use super::yaku::Yaku;

    #[derive(Debug, Clone, PartialEq, Eq)]
    /// Represents the named point limits for high-scoring hands.
    pub enum HandLimit {
        Mangan,    // 満貫
        Haneman,   // 跳満
        Baiman,    // 倍満
        Sanbaiman, // 三倍満
        Yakuman,   // 役満 (13han+)
                   // Kazoe Yakuman (counted Yakuman) can be represented by
                   // han >= 13 and mapping to HandLimit::Yakuman.
    }

    // --- NEW: Implement Display for HandLimit ---
    impl fmt::Display for HandLimit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[derive(Debug, Clone)]
    /// Represents the complete scoring result for a winning hand.
    pub struct AgariResult {
        pub han: u8,            // 飜 (Han count)
        pub fu: u8,             // 符 (Fu count)
        pub yaku_list: Vec<Yaku>, // List of all yaku and dora achieved
        pub num_akadora: u8,    // --- NEW: Added for display ---

        /// The named limit, if one is reached.
        pub limit_name: Option<HandLimit>,

        /// Base points. For ron, this is the total.
        /// For tsumo, this is the non-dealer payment.
        pub base_points: u32,

        // Payment fields are split for clarity in Tsumo calculations
        /// For non-dealer tsumo, this is the dealer's payment.
        /// For dealer tsumo, this is the (identical) payment from each non-dealer.
        /// --- NOTE: This value is *before* honba. ---
        pub oya_payment: u32,

        /// For non-dealer tsumo, this is the payment from other non-dealers.
        /// For dealer tsumo, this is 0.
        /// --- NOTE: This value is *before* honba. ---
        pub ko_payment: u32,

        /// Total points received by the winner, including honba.
        pub total_payment: u32,

        // --- MODIFIED: NEW FIELDS FOR DISPLAY ---
        /// Honba count, for displaying payments correctly.
        pub honba: u8,
        /// How the hand was won.
        pub agari_type: AgariType,
        /// Whether the winner was Oya (dealer).
        pub is_oya: bool,
    }

    // --- MODIFIED: Implement Display for AgariResult ---
    impl fmt::Display for AgariResult {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "========== Score Result ==========")?;

            // 1. Total Score
            writeln!(f, "Total Payment: {}", self.total_payment)?;

            // 2. Han / Fu & 4. Limit Name
            if let Some(limit) = &self.limit_name {
                if limit == &HandLimit::Yakuman {
                    let num_yakuman = self.han / 13;
                    if num_yakuman > 1 {
                        writeln!(f, "\n{} x{}", limit, num_yakuman)?;
                    } else {
                        writeln!(f, "\n{}", limit)?;
                    }
                } else {
                    writeln!(f, "\n{} ({} Fu, {} Han)", limit, self.fu, self.han)?;
                }
            } else {
                writeln!(f, "\n{} Fu, {} Han", self.fu, self.han)?;
            }

            // 3. Yaku List
            writeln!(f, "\n--- Yaku List ---")?;
            let mut dora_count = 0;
            let mut uradora_count = 0;
            let mut akadora_count = 0;

            if self.yaku_list.is_empty() {
                writeln!(f, "(No yaku)")?;
            }

            for yaku in &self.yaku_list {
                match yaku {
                    Yaku::Dora => dora_count += 1,
                    Yaku::UraDora => uradora_count += 1,
                    Yaku::AkaDora => akadora_count += 1,
                    _ => writeln!(f, "- {}", yaku)?,
                }
            }
            if dora_count > 0 {
                writeln!(f, "- Dora {}", dora_count)?;
            }
            if uradora_count > 0 {
                writeln!(f, "- UraDora {}", uradora_count)?;
            }
            if akadora_count > 0 {
                writeln!(f, "- AkaDora {}", akadora_count)?;
            }

            // 5. Payment Breakdown
            writeln!(f, "\n--- Payment Breakdown ---")?;
            let tsumo_bonus = self.honba as u32 * 100;
            let ron_bonus = self.honba as u32 * 300;

            match (self.is_oya, self.agari_type) {
                // Oya Tsumo
                (true, AgariType::Tsumo) => {
                    let payment_per_player = self.oya_payment + tsumo_bonus;
                    writeln!(f, "Oya Tsumo win:")?;
                    writeln!(
                        f,
                        "  Each Non-Dealer pays: {} ({} base + {} honba)",
                        payment_per_player, self.oya_payment, tsumo_bonus
                    )?;
                }
                // Ko Tsumo
                (false, AgariType::Tsumo) => {
                    let oya_pay = self.oya_payment + tsumo_bonus;
                    let ko_pay = self.ko_payment + tsumo_bonus;
                    writeln!(f, "Ko Tsumo win:")?;
                    writeln!(
                        f,
                        "  Oya (Dealer) pays: {} ({} base + {} honba)",
                        oya_pay, self.oya_payment, tsumo_bonus
                    )?;
                    writeln!(
                        f,
                        "  Other Non-Dealers pay: {} ({} base + {} honba)",
                        ko_pay, self.ko_payment, tsumo_bonus
                    )?;
                }
                // Oya Ron
                (true, AgariType::Ron) => {
                    let base_payment = self.total_payment - ron_bonus;
                    writeln!(f, "Oya Ron win:")?;
                    writeln!(
                        f,
                        "  Discarder pays: {} ({} base + {} honba)",
                        self.total_payment, base_payment, ron_bonus
                    )?;
                }
                // Ko Ron
                (false, AgariType::Ron) => {
                    let base_payment = self.total_payment - ron_bonus;
                    writeln!(f, "Ko Ron win:")?;
                    writeln!(
                        f,
                        "  Discarder pays: {} ({} base + {} honba)",
                        self.total_payment, base_payment, ron_bonus
                    )?;
                }
            }

            writeln!(f, "==================================")?;
            Ok(())
        }
    }
}

/// # User Input Module
/// This module handles user input for the application.
pub mod input {
    // --- REFACTOR: Import context structs ---
    use super::game::{AgariType, GameContext, PlayerContext};
    use super::hand::MentsuType;
    use super::tiles::Hai;

    /// Represents a meld that was made "open" (stolen from another player).
    /// This is used to distinguish from concealed melds in the hand.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OpenMeldInput {
        /// The type of meld.
        /// Koutsu (Pon), Kantsu (Daiminkan/Shouminkan), or Shuntsu (Chi).
        pub mentsu_type: MentsuType,

        /// Any single tile that uniquely identifies the meld.
        /// e.g., for a 1-2-3 Shuntsu (Chi) of Manzu, `Hai::Suhai(1, Suhai::Manzu)` is fine.
        /// e.g., for a 7-7-7 Koutsu (Pon) of Pinzu, `Hai::Suhai(7, Suhai::Pinzu)` is fine.
        /// The parser will use this to find the full meld in the main `hand_tiles`.
        pub representative_tile: Hai,
    }

    /// Holds all the information provided by the user, ready to be
    /// processed by the hand organizer and score calculator.
    #[derive(Debug, Clone)]
    pub struct UserInput {
        // --- 1. Hand Composition ---
        /// The complete, unorganized winning hand (14-18 tiles).
        /// This Vec should include all tiles from open melds and closed kans.
        pub hand_tiles: Vec<Hai>,

        /// The single tile that completed the hand (Tsumo or Ron).
        /// This tile must also be present in `hand_tiles`.
        pub winning_tile: Hai,

        /// A list of all open melds (Chi, Pon, Daiminkan, Shouminkan).
        /// This tells the parser which parts of `hand_tiles` are not concealed.
        /// An empty list means the hand is fully concealed (Menzen).
        pub open_melds: Vec<OpenMeldInput>,

        /// A list of all closed kans (Ankan).
        /// This is needed to distinguish a 4-of-a-kind Ankan from
        /// a 3-of-a-kind Koutsu + a spare tile, which is crucial for fu calculation
        /// and yaku like Sanankou/Suuankou.
        /// Only the representative tile is needed (e.g., `Hai::Suhai(1, Suhai::Manzu)` for four 1-mans).
        pub closed_kans: Vec<Hai>,

        // --- REFACTOR: Embed context structs to remove redundancy ---
        // All the duplicated fields for game state have been moved
        // into these two structs, matching the yaku_checker's interface.
        pub player_context: PlayerContext,
        pub game_context: GameContext,
        pub agari_type: AgariType,
    }
}