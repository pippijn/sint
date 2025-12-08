use crate::logic::cards::behavior::CardBehavior;

pub struct C27WailingAlarm;

impl CardBehavior for C27WailingAlarm {
    // Effect: No Bonuses.
    // - Extinguisher removes only 1 (Default behavior is 1, so no change unless we implement Extinguisher item logic).
    // - Runner does not work (Item logic).
    // - Shoot together: Roll die. (Default is roll die).
    // Basically enforces defaults.
    // Since we haven't implemented Item Bonuses yet, this card effectively does nothing but exists.
}
