use pwm_speaker::{self, Speaker};

pub struct Alarm {
    speaker: Speaker,
    playing: bool,
    song: pwm_speaker::songs::MsEvents,
}
impl Alarm {
    pub fn new(speaker: Speaker) -> Alarm {
        Alarm {
            speaker,
            playing: false,
            song: pwm_speaker::songs::MARIO_THEME_INTRO.events().ms_events(),
        }
    }
    pub fn play(&mut self) {
        self.song = pwm_speaker::songs::MARIO_THEME_INTRO.events().ms_events();
        self.playing = true;
        self.speaker.unmute();
    }
    pub fn stop(&mut self) {
        self.playing = false;
        self.speaker.rest();
        self.speaker.mute();
    }
    pub fn poll(&mut self) {
        if !self.playing {
            return;
        }

        use pwm_speaker::songs::MsEvent::*;
        match self.song.next() {
            Some(BeginNote { pitch }) => self.speaker.play(pitch),
            Some(EndNote) => self.speaker.rest(),
            Some(Wait) => (),
            None => self.stop(),
        }
    }
}
