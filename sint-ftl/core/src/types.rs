use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- ID Aliases ---
pub type PlayerId = String;
pub type RoomId = u32;

// --- Top Level State ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GameState {
    /// Incremental version for P2P sync
    pub sequence_id: u64,
    /// Random seed for the next deterministic event
    pub rng_seed: u64,
    /// Current game phase
    pub phase: GamePhase,
    /// Total Action Points available to the team (shared pool logic or just tracking)
    /// In v2 rules: Each player has 2 AP. This might track "rounds" or be unused.
    /// We'll track the round number.
    pub turn_count: u32,
    /// Ship Health (starts at 20)
    pub hull_integrity: i32,

    /// The Map
    pub map: GameMap,

    /// The Players
    pub players: HashMap<PlayerId, Player>,

    /// The Enemy (Boss)
    pub enemy: Enemy,

    /// Chat History (Event Sourcing derived or stored)
    pub chat_log: Vec<ChatMessage>,

    // --- Temporary Status Flags (Reset each round) ---
    pub shields_active: bool,
    pub evasion_active: bool,

    /// Proposed Actions (for Tactical Planning phase)
    pub proposal_queue: Vec<ProposedAction>,

    /// Active "Situation" cards
    pub active_situations: Vec<Card>,

    /// The card drawn this turn (Flash or Situation) for display in MorningReport
    pub latest_event: Option<Card>,

    /// The Draw Deck
    pub deck: Vec<Card>,
    /// The Discard Pile
    pub discard: Vec<Card>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum GamePhase {
    Lobby, // Waiting for players
    Setup,
    MorningReport,    // Card draw
    EnemyTelegraph,   // Enemy reveals intent
    TacticalPlanning, // Players propose/commit actions
    Execution,        // Actions resolve
    EnemyAction,      // Enemy attacks, fire spreads
    GameOver,
}

// --- Map & Rooms ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GameMap {
    pub rooms: HashMap<RoomId, Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub system: Option<SystemType>,
    pub hazards: Vec<HazardType>,
    pub items: Vec<ItemType>,
    pub neighbors: Vec<RoomId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SystemType {
    Bridge,    // Room 9
    Engine,    // Room 5
    Kitchen,   // Room 6
    Cannons,   // Room 8
    Sickbay,   // Room 10
    Bow,       // Room 2
    Cargo,     // Room 4
    Dormitory, // Room 3
    Storage,   // Room 11
    Hallway,   // Room 7 (Transit only, usually)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum HazardType {
    Fire,
    Water,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ItemType {
    Peppernut,
    Extinguisher,
    Keychain,
    Wheelbarrow,
    Mitre,
}

// --- Player ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub room_id: RoomId,
    pub hp: i32, // Max 3
    pub ap: i32, // Max 2
    pub inventory: Vec<ItemType>,
    pub status: Vec<PlayerStatus>,
    /// Has this player voted "Ready" for the current proposal batch?
    pub is_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlayerStatus {
    Fainted,
    Silenced,
}

// --- Enemy ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    /// What the enemy plans to do next (Telegraphing)
    pub next_attack: Option<EnemyAttack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EnemyAttack {
    pub target_room: RoomId,
    pub effect: AttackEffect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AttackEffect {
    Fireball, // Spawns Fire
    Leak,     // Spawns Water
    Boarding, // Spawns Blockade?
    Special(String),
}

// --- Actions & Events ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub sender: PlayerId,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProposedAction {
    pub id: String, // UUID
    pub player_id: PlayerId,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "payload")]
pub enum Action {
    /// Move to an adjacent room (Costs 1 AP)
    Move { to_room: RoomId },

    /// Kitchen: Create Peppernuts (Costs 1 AP)
    Bake,
    /// Cannons: Load & Fire at Enemy (Costs 1 AP + 1 Peppernut)
    Shoot,
    /// Bridge: Block next damage (Costs 2 AP)
    RaiseShields,
    /// Engine: Dodge next attack (Costs 2 AP)
    EvasiveManeuvers,

    /// Generic Interact (e.g. solve card, use button)
    Interact,

    /// Remove 1 Fire token (Costs 1 AP)
    Extinguish,
    /// Remove 1 Water token (Costs 1 AP)
    Repair,
    /// Give an item to another player in the same/adjacent room (Costs 1 AP)
    Throw {
        target_player: PlayerId,
        item_index: usize,
    },
    /// Pick up an item from the floor (Costs 1 AP)
    PickUp { item_type: ItemType },
    /// Drop an item to the floor (Free)
    Drop { item_index: usize },
    /// Revive a Fainted player in the same room (Costs 1 AP)
    Revive { target_player: PlayerId },

    /// Send a chat message (Free)
    Chat { message: String },
    /// Toggle "Ready" status for the batch execution
    VoteReady { ready: bool },
    /// Skip remaining AP for this round
    Pass,
    /// Join the game dynamically
    Join { name: String },
    /// Set the player name (Only in Lobby)
    SetName { name: String },
    /// Receive a full state dump from a peer
    FullSync { state_json: String },
    /// Undo a queued proposed action
    Undo { action_id: String },
}

// --- Cards ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum CardId {
    AfternoonNap,
    StaticNoise,
    SeagullAttack,
    SlipperyDeck,
    PeppernutRain,
    HighWaves,
    CostumeParty,
    MicePlague,
    ShortCircuit,
    Leak,
    Mutiny,
    FogBank,
    AnchorStuck,
    JammedCannon,
    ManOverboard,
    StrongHeadwind,
    Listing,
    CloggedPipe,
    AttackWave,
    SingASong,
    NoLight,
    LuckyDip,
    Panic,
    Seasick,
    WailingAlarm,
    AnchorLoose,
    Rudderless,
    BigLeak,
    Blockade,
    WeirdGifts,
    FluWave,
    MonsterDough,
    Stowaway,
    TurboMode,
    Recipe,
    GoldenNut,
    TheStaff,
    StickyFloor,
    Present,
    TheBook,
    SugarRush,
    WheelClamp,
    ShoeSetting,
    Amerigo,
    Tantrum,
    SilentForce,
    FallingGift,
    Overheating,
    FalseNote,
    HighPressure,
    LightsOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Card {
    pub id: CardId,
    pub title: String,
    pub description: String,
    pub card_type: CardType,
    /// If the card offers a choice (Dilemma)
    pub options: Vec<CardOption>,
    /// Solution for Situations/Timebombs
    pub solution: Option<CardSolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CardType {
    Flash,                         // Instant effect
    Situation,                     // Persistent negative effect
    Timebomb { rounds_left: u32 }, // Countdown
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CardOption {
    pub text: String,
    pub effect: EffectType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EffectType {
    None,
    DamageHull(i32),
    LoseResource(ItemType, u32),
    MovePlayer(String, RoomId),
    SpawnHazard(RoomId, HazardType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CardSolution {
    pub room_id: Option<RoomId>, // Where to solve it
    pub ap_cost: u32,
    pub item_cost: Option<ItemType>,
    pub required_players: u32,
}
