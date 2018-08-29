use pwm_speaker::{songs, Speaker};

struct IterNb<I> {
    iter: I,
    cur: I,
    nb: usize,
}
impl<I: Clone> IterNb<I> {
    fn new(nb: usize, iter: I) -> Self {
        Self {
            cur: iter.clone(),
            iter,
            nb,
        }
    }
}
impl<I: Iterator + Clone> Iterator for IterNb<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.cur.next(), self.nb) {
                (Some(i), _) => return Some(i),
                (None, 0) => return None,
                (None, _) => {
                    self.nb -= 1;
                    self.cur = self.iter.clone();
                }
            }
        }
    }
}

pub struct Alarm {
    speaker: Speaker,
    playing: bool,
    song: IterNb<songs::MsEvents>,
}
impl Alarm {
    pub fn new(speaker: Speaker) -> Alarm {
        Alarm {
            speaker,
            playing: false,
            song: IterNb::new(0, songs::MARIO_THEME_INTRO.ms_events()),
        }
    }
    pub fn play(&mut self, song: &'static songs::Score, nb_sec: u32) {
        let song_ms = song.ms_duration();
        let nb = if song_ms == 0 {
            0
        } else {
            nb_sec * 1000 / song_ms
        };
        self.song = IterNb::new(nb as usize, song.ms_events());
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
