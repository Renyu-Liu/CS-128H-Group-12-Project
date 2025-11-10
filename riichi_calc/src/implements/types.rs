#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit { Man, Pin, Sou }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Honor { Wind(Wind), Dragon(Dragon) }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wind { East, South, West, North }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dragon { White, Green, Red }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Number(u8, Suit), // u8 from 1-9
    Honor(Honor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeldType { Sequence, Triplet, Kan }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitType {
    TwoSided, // Ryanmen
    Pair,     // Tanki
    Edge,     // Penchan
    Closed,   // Kanchan
    Single,   // Shanpon
    KokushiSingle, // For 13 orphans
    KokushiThirteenSided, // For 13 orphans
}

#[derive(Debug, Clone, Copy)]
pub struct Meld {
    pub meld_type: MeldType,
    pub is_open: bool,
    pub tiles: [Tile; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct Hand {
    pub tiles: [u8; 34],
}

#[derive(Debug, Clone)]
pub struct WinningHand {
    pub melds: [Meld; 4],
    pub pair: (Tile, Tile),
    pub winning_tile: Tile,
    pub wait_type: WaitType,
}

// This enum allows us to store different valid hand structures.
#[derive(Debug, Clone)]
pub enum HandStructure {
    /// The standard 4 melds + 1 pair hand
    Standard(WinningHand),
    
    /// The 7 pairs (Chiitoitsu) hand
    SevenPairs {
        pairs: [Tile; 7],
        winning_tile: Tile,
        wait_type: WaitType, // Will always be WaitType::Pair
    },
    
    /// The 13 orphans (Kokushi Musou) hand
    ThirteenOrphans {
        pair_tile: Tile,
        winning_tile: Tile,
        wait_type: WaitType, // KokushiSingle or KokushiThirteenSided
    },
}

#[derive(Debug, Clone)]
pub struct GameConditions {
    pub seat_wind: Wind,
    pub prevalent_wind: Wind,
    pub is_riichi: bool,
    pub is_ippatsu: bool,
    pub is_tsumo: bool,
    pub is_closed_hand: bool,
    pub dora_indicators: Vec<Tile>,
    pub ura_dora_indicators: Vec<Tile>,
    // ... and booleans for Haitei, Rinshan, etc.
}

#[derive(Debug, Clone)]
pub struct RawHandInput {
    pub tiles: Vec<Tile>,
    pub winning_tile: Tile,
    pub open_melds: Vec<Meld>,
    pub game_conditions: GameConditions,
}

// OrganizedHand now holds a HandStructure enum instead of just a WinningHand.
#[derive(Debug, Clone)]
pub struct OrganizedHand {
    pub hand_structure: HandStructure, // Replaced winning_hand
    pub game_conditions: GameConditions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Yaku {
    Riichi(u8),
    Tanyao(u8),
    Pinfu(u8),
    Honitsu(u8),
    Chiitoitsu(u8),     // 2 Han (for 7 pairs)
    Daisangen(u8),
    KokushiMusou(u8), // Yakuman (for 13 orphans)
    // ... etc.
}

#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub han: u8,
    pub fu: u8,
    pub points: u32,
    pub yaku_list: Vec<Yaku>,
    pub score_name: String,
}