use core::fmt::Write;

use crate::{
    color::PaletteColor,
    ewram_static,
    ewramstring::EwramString,
    screen_text::{ScreenTextManagerStr, WriteTicket},
    static_init::StaticInitSafe,
};

pub struct ScoreManager {
    score: u32,
    new_score: Option<u32>,
    score_handle: Option<WriteTicket>,
}

unsafe impl StaticInitSafe for ScoreManager {
    // Uses default no-op init
}

ewram_static!(HIGHSCORE_STR: EwramString<10> = EwramString::new());
ewram_static!(Score: ScoreManager = ScoreManager::new());

impl ScoreManager {
    pub const fn new() -> Self {
        ScoreManager {
            score: 0,
            new_score: Some(0),
            score_handle: None,
        }
    }

    pub fn update_score(score: u32) {
        let manager = Score.get_or_init();
        manager.new_score = Some(score);
    }

    pub fn add_to_score(score: u32) {
        let manager = Score.get_or_init();
        let mut cur_score = manager.score + score;
        if let Some(ok) = manager.new_score {
            cur_score += ok;
        }
        manager.new_score = Some(cur_score);
    }

    pub fn tick(screen_text: &mut ScreenTextManagerStr) {
        let manager = Score.get_or_init();
        let Some(score) = manager.new_score.take() else {
            return;
        };
        manager.score_handle.take();
        let str = HIGHSCORE_STR.get_or_init();
        str.clear();
        write!(str, "{}", score).unwrap();
        manager.score = score;
        manager.score_handle =
            screen_text.write_text(2, str.as_str(), (0, 0), PaletteColor::Cyan, false);
    }
}
