use std::collections::HashMap;
use std::sync::OnceLock;
use crate::types::CardId;
use super::behavior::{CardBehavior, NoOpBehavior};
use super::impls::{
    c01_afternoon::C01AfternoonNap,
    c02_static::C02StaticNoise,
    c03_seagull::C03SeagullAttack,
    c04_slippery::C04SlipperyDeck,
    c05_rain::C05PeppernutRain,
    c06_waves::C06HighWaves,
    c07_costume::C07CostumeParty,
    c08_mice::C08MicePlague,
    c09_short_circuit::C09ShortCircuit,
    c10_leak::C10Leak,
    c11_mutiny::C11Mutiny,
    c12_fog::C12FogBank,
    c13_anchor::C13AnchorStuck,
    c14_jammed::C14JammedCannon,
    c15_man_overboard::C15ManOverboard,
    c16_headwind::C16StrongHeadwind,
    c17_listing::C17Listing,
    c18_clogged::C18CloggedPipe,
    c19_attack_wave::C19AttackWave,
    c21_sing::C21SingASong,
    c23_no_light::C23NoLight,
    c24_lucky_dip::C24LuckyDip,
    c25_panic::C25Panic,
    c26_seasick::C26Seasick,
    c27_alarm::C27WailingAlarm,
    c28_anchor_loose::C28AnchorLoose,
    c29_rudderless::C29Rudderless,
    c30_big_leak::C30BigLeak,
    c31_blockade::C31Blockade,
    c32_weird_gifts::C32WeirdGifts,
    c33_flu::C33FluWave,
    c34_dough::C34MonsterDough,
    c35_stowaway::C35Stowaway,
    c36_turbo::C36TurboMode,
    c37_recipe::C37Recipe,
    c38_golden_nut::C38GoldenNut,
    c39_staff::C39TheStaff,
    c40_sticky::C40StickyFloor,
    c41_present::C41Present,
    c42_book::C42TheBook,
    c43_sugar::C43SugarRush,
    c44_clamp::C44WheelClamp,
    c45_shoe::C45ShoeSetting,
};

// Registry Type
type BehaviorMap = HashMap<CardId, Box<dyn CardBehavior>>;

// Global Registry
static REGISTRY: OnceLock<BehaviorMap> = OnceLock::new();

pub fn get_behavior(card_id: CardId) -> &'static dyn CardBehavior {
    REGISTRY.get_or_init(init_registry)
        .get(&card_id)
        .map(|b| b.as_ref())
        .unwrap_or(&NoOpBehavior)
}

fn init_registry() -> BehaviorMap {
    let mut m: BehaviorMap = HashMap::new();
    m.insert(CardId::AfternoonNap, Box::new(C01AfternoonNap));
    m.insert(CardId::StaticNoise, Box::new(C02StaticNoise));
    m.insert(CardId::SeagullAttack, Box::new(C03SeagullAttack));
    m.insert(CardId::SlipperyDeck, Box::new(C04SlipperyDeck));
    m.insert(CardId::PeppernutRain, Box::new(C05PeppernutRain));
    m.insert(CardId::HighWaves, Box::new(C06HighWaves));
    m.insert(CardId::CostumeParty, Box::new(C07CostumeParty));
    m.insert(CardId::MicePlague, Box::new(C08MicePlague));
    m.insert(CardId::ShortCircuit, Box::new(C09ShortCircuit));
    m.insert(CardId::Leak, Box::new(C10Leak));
    m.insert(CardId::Mutiny, Box::new(C11Mutiny));
    m.insert(CardId::FogBank, Box::new(C12FogBank));
    m.insert(CardId::AnchorStuck, Box::new(C13AnchorStuck));
    m.insert(CardId::JammedCannon, Box::new(C14JammedCannon));
    m.insert(CardId::ManOverboard, Box::new(C15ManOverboard));
    m.insert(CardId::StrongHeadwind, Box::new(C16StrongHeadwind));
    m.insert(CardId::Listing, Box::new(C17Listing));
    m.insert(CardId::CloggedPipe, Box::new(C18CloggedPipe));
    m.insert(CardId::AttackWave, Box::new(C19AttackWave));
    m.insert(CardId::SingASong, Box::new(C21SingASong));
    m.insert(CardId::NoLight, Box::new(C23NoLight));
    m.insert(CardId::LuckyDip, Box::new(C24LuckyDip));
    m.insert(CardId::Panic, Box::new(C25Panic));
    m.insert(CardId::Seasick, Box::new(C26Seasick));
    m.insert(CardId::WailingAlarm, Box::new(C27WailingAlarm));
    m.insert(CardId::AnchorLoose, Box::new(C28AnchorLoose));
    m.insert(CardId::Rudderless, Box::new(C29Rudderless));
    m.insert(CardId::BigLeak, Box::new(C30BigLeak));
    m.insert(CardId::Blockade, Box::new(C31Blockade));
    m.insert(CardId::WeirdGifts, Box::new(C32WeirdGifts));
    m.insert(CardId::FluWave, Box::new(C33FluWave));
    m.insert(CardId::MonsterDough, Box::new(C34MonsterDough));
    m.insert(CardId::Stowaway, Box::new(C35Stowaway));
    m.insert(CardId::TurboMode, Box::new(C36TurboMode));
    m.insert(CardId::Recipe, Box::new(C37Recipe));
    m.insert(CardId::GoldenNut, Box::new(C38GoldenNut));
    m.insert(CardId::TheStaff, Box::new(C39TheStaff));
    m.insert(CardId::StickyFloor, Box::new(C40StickyFloor));
    m.insert(CardId::Present, Box::new(C41Present));
    m.insert(CardId::TheBook, Box::new(C42TheBook));
    m.insert(CardId::SugarRush, Box::new(C43SugarRush));
    m.insert(CardId::WheelClamp, Box::new(C44WheelClamp));
    m.insert(CardId::ShoeSetting, Box::new(C45ShoeSetting));
    m
}
