use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use uuid::Uuid;

use crate::field_map::{FieldMap, Identifiable};
use crate::small_map::SmallMap;

// --- ID Aliases ---
pub type PlayerId = String;
pub type RoomId = u32;

pub const MAX_HULL: i32 = 20;

// --- Map Layout ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default, Hash)]
pub enum MapLayout {
    #[default]
    Star,
    Torus,
}

// --- Top Level State ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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
    /// Current Boss Level (0-3)
    pub boss_level: u32,

    /// The Map Layout (Star, Torus, etc.)
    pub layout: MapLayout,

    /// The Map
    pub map: GameMap,

    /// The Players
    pub players: FieldMap<Player>,

    /// The Enemy (Boss)
    pub enemy: Enemy,

    /// Chat History (Event Sourcing derived or stored)
    pub chat_log: Vec<ChatMessage>,

    // --- Temporary Status Flags (Reset each round) ---
    pub shields_active: bool,
    pub evasion_active: bool,

    // Rest Round Flag
    pub is_resting: bool,

    /// Proposed Actions (for Tactical Planning phase)
    #[schemars(with = "Vec<ProposedAction>")]
    pub proposal_queue: SmallVec<[ProposedAction; 8]>,

    /// Active "Situation" cards
    #[schemars(with = "Vec<Card>")]
    pub active_situations: SmallVec<[Card; 4]>,

    /// The card drawn this turn (Flash or Situation) for display in MorningReport
    pub latest_event: Option<Card>,

    /// The Draw Deck
    pub deck: Vec<CardId>,
    /// The Discard Pile
    pub discard: Vec<CardId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum GamePhase {
    Lobby, // Waiting for players
    Setup,
    MorningReport,    // Card draw
    EnemyTelegraph,   // Enemy reveals intent
    TacticalPlanning, // Players propose/commit actions
    Execution,        // Actions resolve
    EnemyAction,      // Enemy attacks, fire spreads
    GameOver,
    Victory,
}

// --- Map & Rooms ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GameMap {
    pub rooms: SmallMap<RoomId, Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Room {
    pub id: RoomId,
    pub name: RoomName,
    pub system: Option<SystemType>,
    #[schemars(with = "Vec<HazardType>")]
    pub hazards: SmallVec<[HazardType; 4]>,
    #[schemars(with = "Vec<ItemType>")]
    pub items: SmallVec<[ItemType; 4]>,
    #[schemars(with = "Vec<RoomId>")]
    pub neighbors: SmallVec<[RoomId; 8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum RoomName {
    CentralHallway,
    Bow,
    Dormitory,
    Cargo,
    Engine,
    Kitchen,
    Cannons,
    Bridge,
    Sickbay,
    Storage,
    CorridorA,
    CorridorB,
    CorridorC,
}

impl RoomName {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoomName::CentralHallway => "Central Hallway",
            RoomName::Bow => "The Bow",
            RoomName::Dormitory => "Dormitory",
            RoomName::Cargo => "Cargo",
            RoomName::Engine => "Engine",
            RoomName::Kitchen => "Kitchen",
            RoomName::Cannons => "Cannons",
            RoomName::Bridge => "Bridge",
            RoomName::Sickbay => "Sickbay",
            RoomName::Storage => "Storage",
            RoomName::CorridorA => "Corridor A",
            RoomName::CorridorB => "Corridor B",
            RoomName::CorridorC => "Corridor C",
        }
    }
}

impl std::fmt::Display for RoomName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum SystemType {
    Bow,       // 2
    Dormitory, // 3
    Cargo,     // 4
    Engine,    // 5
    Kitchen,   // 6
    Cannons,   // 7
    Bridge,    // 8
    Sickbay,   // 9
    Storage,   // 10
}

impl SystemType {
    pub fn as_u32(&self) -> u32 {
        match self {
            SystemType::Bow => 2,
            SystemType::Dormitory => 3,
            SystemType::Cargo => 4,
            SystemType::Engine => 5,
            SystemType::Kitchen => 6,
            SystemType::Cannons => 7,
            SystemType::Bridge => 8,
            SystemType::Sickbay => 9,
            SystemType::Storage => 10,
        }
    }

    pub fn from_u32(id: u32) -> Option<Self> {
        match id {
            2 => Some(SystemType::Bow),
            3 => Some(SystemType::Dormitory),
            4 => Some(SystemType::Cargo),
            5 => Some(SystemType::Engine),
            6 => Some(SystemType::Kitchen),
            7 => Some(SystemType::Cannons),
            8 => Some(SystemType::Bridge),
            9 => Some(SystemType::Sickbay),
            10 => Some(SystemType::Storage),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum HazardType {
    Fire,
    Water,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash, PartialOrd, Ord,
)]
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
    #[schemars(with = "Vec<ItemType>")]
    pub inventory: SmallVec<[ItemType; 5]>,
    #[schemars(with = "Vec<PlayerStatus>")]
    pub status: SmallVec<[PlayerStatus; 2]>,
    /// Has this player voted "Ready" for the current proposal batch?
    pub is_ready: bool,
}

impl Identifiable for Player {
    type Id = PlayerId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum PlayerStatus {
    Fainted,
    Silenced,
}

// --- Enemy ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum EnemyState {
    Active,
    Defeated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub state: EnemyState,
    /// What the enemy plans to do next (Telegraphing)
    pub next_attack: Option<EnemyAttack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct EnemyAttack {
    pub target_room: RoomId,
    pub target_system: Option<SystemType>,
    pub effect: AttackEffect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum AttackEffect {
    Fireball, // Spawns Fire
    Leak,     // Spawns Water
    Boarding, // Spawns Blockade?
    Hidden,   // Masked by Fog or other effects
    Miss,     // Lucky Miss (Roll 11/12)
    Special(String),
}

// --- Actions & Events ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct ChatMessage {
    pub sender: PlayerId,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct ProposedAction {
    pub id: Uuid, // UUID
    pub player_id: PlayerId,
    pub action: GameAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct PlayerEvent {
    pub id: Uuid, // UUID
    pub player_id: PlayerId,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(untagged)]
pub enum Action {
    Game(GameAction),
    Meta(MetaAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(tag = "type", content = "payload")]
pub enum GameAction {
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

    /// Bow: Reveal the next event card (Costs 1 AP)
    Lookout,
    /// Sickbay: Restore 1 HP to self or adjacent player (Costs 1 AP)
    FirstAid { target_player: PlayerId },

    /// Send a chat message (Free)
    Chat { message: String },
    /// Toggle "Ready" status for the batch execution
    VoteReady { ready: bool },
    /// Skip remaining AP for this round
    Pass,
    /// Undo a queued proposed action
    Undo { action_id: Uuid },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(tag = "type", content = "payload")]
pub enum MetaAction {
    /// Join the game dynamically
    Join { name: String },
    /// Set the player name (Only in Lobby)
    SetName { name: String },
    /// Set the map layout (Only in Lobby)
    SetMapLayout { layout: MapLayout },
    /// Receive a full state dump from a peer
    FullSync { state_json: String },
}

// --- Cards ---

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, PartialOrd, Ord,
)]
pub enum CardId {
    // keep-sorted start
    AfternoonNap,
    Amerigo,
    AnchorLoose,
    AnchorStuck,
    AttackWave,
    BigLeak,
    Blockade,
    CloggedPipe,
    CostumeParty,
    FallingGift,
    FalseNote,
    FluWave,
    FogBank,
    GoldenNut,
    HighPressure,
    HighWaves,
    JammedCannon,
    Leak,
    LightsOut,
    Listing,
    LuckyDip,
    ManOverboard,
    MicePlague,
    MonsterDough,
    Mutiny,
    NoLight,
    Overheating,
    Panic,
    PeppernutRain,
    Present,
    Recipe,
    Rudderless,
    SeagullAttack,
    Seasick,
    ShoeSetting,
    ShortCircuit,
    SilentForce,
    SingASong,
    SlipperyDeck,
    StaticNoise,
    StickyFloor,
    Stowaway,
    StrongHeadwind,
    SugarRush,
    TheBook,
    TheStaff,
    TurboMode,
    WailingAlarm,
    WeirdGifts,
    WheelClamp,
    // keep-sorted end
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct Card {
    pub id: CardId,
    pub title: String,
    pub description: String,
    pub card_type: CardType,
    /// If the card offers a choice (Dilemma)
    #[schemars(with = "Vec<CardOption>")]
    pub options: SmallVec<[CardOption; 2]>,
    /// Solution for Situations/Timebombs
    pub solution: Option<CardSolution>,
    /// The player targeted or affected by this card (e.g. The Reader)
    pub affected_player: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum CardType {
    Flash,                         // Instant effect
    Situation,                     // Persistent negative effect
    Timebomb { rounds_left: u32 }, // Countdown
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct CardOption {
    pub text: String,
    pub effect: EffectType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash, Default)]
pub enum CardSentiment {
    #[default]
    Negative,
    Neutral,
    Positive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum EffectType {
    None,
    DamageHull(i32),
    LoseResource(ItemType, u32),
    MovePlayer(String, RoomId),
    SpawnHazard(RoomId, HazardType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Hash)]
pub struct CardSolution {
    pub target_system: Option<SystemType>, // Where to solve it (None = Any/Special)
    pub ap_cost: u32,
    pub item_cost: Option<ItemType>,
    pub required_players: u32,
}
