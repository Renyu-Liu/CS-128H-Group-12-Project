pub enum Suit { Man, Pin, Sou }
pub enum Honor { Wind(Wind), Dragon(Dragon) }
pub enum Wind { East, South, West, North }
pub enum Dragon { White, Green, Red }
pub enum Tile {
    Number(u8, Suit), // u8 from 1-9
    Honor(Honor),
}
pub struct Meld {
    pub meld_type: MeldType,
    pub is_open: bool,
    pub tiles: [Tile; 4], // Use 4, Kants just have a different type
}
pub enum MeldType { Sequence, Triplet, Kan }
// An array where the index 0-33 corresponds to a unique tile.
// e.g., 0-8=Man, 9-17=Pin, 18-26=Sou, 27-33=Honors
// The value at the index is the count of that tile (0-4).
pub enum WaitType { TwoSided, Pair, Edge, Closed, Single }
pub struct Hand {
    pub tiles: [u8; 34],
}
pub struct WinningHand {
    pub melds: [Meld; 4],
    pub pair: (Tile, Tile),
    pub winning_tile: Tile,
    pub wait_type: WaitType, // e.g., TwoSided, Pair, Edge
}
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
// An enum listing every Yaku, storing its Han value.
pub enum Yaku {
    Riichi(u8),       // 1 Han
    Tanyao(u8),       // 1 Han
    Pinfu(u8),       // 1 Han
    Honitsu(u8),     // 3 Han (2 if open)
    Daisangen(u8),        // Yakuman
    // ... etc.
}
// The final output of our library.
pub struct ScoreResult {
    pub han: u8,
    pub fu: u8,
    pub points: u32,
    pub yaku_list: Vec<Yaku>,
    pub score_name: String, // e.g., "Mangan", "Haneman"
}