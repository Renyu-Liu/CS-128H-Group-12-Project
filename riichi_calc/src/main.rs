mod implements;
use implements::*;
use implements::tiles::{Hai, Suhai, Kaze};
use implements::game::{AgariType, GameContext, PlayerContext};
use implements::input::UserInput;
use implements::scoring::{AgariResult};


/// A helper function to create the example `UserInput`.
fn create_example_hand_input() -> UserInput {
    // --- 1. Define the Hand Composition ---
    // This is a 14-tile Tanyao Pinfu hand, won on a Tsumo.
    // Hand: 234m 567m 345p 678p 44s
    // Wait was on 5p or 8p (Ryanmen).
    // The player Tsumo'd the 8p.
    let hand_tiles = vec![
        Hai::Suhai(2, Suhai::Manzu),
        Hai::Suhai(3, Suhai::Manzu),
        Hai::Suhai(4, Suhai::Manzu),
        Hai::Suhai(5, Suhai::Manzu),
        Hai::Suhai(6, Suhai::Manzu),
        Hai::Suhai(7, Suhai::Manzu), // This will be UraDora
        Hai::Suhai(3, Suhai::Pinzu), // This will be Dora
        Hai::Suhai(4, Suhai::Pinzu),
        Hai::Suhai(5, Suhai::Pinzu), // This will be AkaDora
        Hai::Suhai(6, Suhai::Pinzu),
        Hai::Suhai(7, Suhai::Pinzu),
        Hai::Suhai(8, Suhai::Pinzu), // This is the winning tile
        Hai::Suhai(4, Suhai::Souzu),
        Hai::Suhai(4, Suhai::Souzu),
    ];

    let winning_tile = Hai::Suhai(8, Suhai::Pinzu);

    // This hand is fully concealed (Menzen).
    let open_melds = Vec::new();
    let closed_kans = Vec::new();

    // --- 2. Define the Player Context ---
    let player_context = PlayerContext {
        jikaze: Kaze::Nan,    // Player is South
        is_oya: false,       // Player is not the dealer (Ko)
        is_riichi: true,     // Player declared Riichi
        is_daburu_riichi: false,
        is_ippatsu: false,
        is_menzen: true,     // Hand is concealed
    };

    // --- 3. Define the Game Context ---
    let game_context = GameContext {
        bakaze: Kaze::Ton,      // Prevalent wind is East
        kyoku: 1,               // East 1
        honba: 1,               // 1 Honba (1 extra counter)
        riichi_bou: 1,          // 1 Riichi stick on the table
        dora_indicators: vec![Hai::Suhai(2, Suhai::Pinzu)], // 2p -> 3p is Dora
        uradora_indicators: vec![Hai::Suhai(6, Suhai::Manzu)], // 6m -> 7m is UraDora
        num_akadora: 1,         // Player has 1 Red Five (the 5p)
        
        // All special yaku are false
        is_tenhou: false,
        is_chiihou: false,
        is_renhou: false,
        is_haitei: false,
        is_houtei: false,
        is_rinshan: false,
        is_chankan: false,
    };

    // --- 4. Define the Agari Type ---
    let agari_type = AgariType::Tsumo;

    // --- 5. Build and return the UserInput struct ---
    UserInput {
        hand_tiles,
        winning_tile,
        open_melds,
        closed_kans,
        player_context,
        game_context,
        agari_type,
    }
}

pub fn calculate_agari(input: &UserInput) -> Result<AgariResult, &'static str> {
    // 1. Get context from input
    // All the types are in scope thanks to `pub use` above.
    let player = &input.player_context;
    let game = &input.game_context;
    let agari_type = input.agari_type;

    // 2. Step 1: Organize Hand
    // The `?` operator will automatically return the Err if `organize_hand` fails.
    let organization = organize_hand(input)?;

    // 3. Step 2: Check Yaku
    // The `?` operator will automatically return the Err if `check_all_yaku` fails.
    let yaku_result = check_all_yaku(organization, player, game, agari_type)?;

    // 4. Step 3: Calculate Final Score
    // This function does not return a Result, as it assumes valid input from Step 2.
    let final_score = calculate_score(yaku_result, player, game, agari_type);

    // 5. Return the successful result
    Ok(final_score)
}

fn main() {
    println!("--- Riichi Mahjong Score Calculator Example ---");

    // 1. Get the raw input
    let user_input = create_example_hand_input();
    
    // 2. Call the NEW single library function
    // We just pass a reference to the input.
    // All the complex steps are now hidden inside `calculate_agari`.
    let calculation_result = calculate_agari(&user_input);

    // 3. Print the final result
    println!("\nCalculating score for hand...\n");

    match calculation_result {
        Ok(final_score) => {
            // This will use the `Display` implementation in `types.rs`
            // that we just updated.
            println!("{}", final_score);
        }
        Err(error_message) => {
            // This handles the "invalid hand" case
            println!("!!! Error calculating score: {} !!!", error_message);
        }
	}
}