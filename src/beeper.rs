use rodio::{
    DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink, Player,
    source::{Source, SquareWave},
};

const PITCH_HZ: f32 = 440.0;
const VOLUME: f32 = 0.1;

pub struct Beeper {
    _sink: MixerDeviceSink,
    player: Player,
}

impl Beeper {
    pub fn open() -> Result<Self, DeviceSinkError> {
        let mut sink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);
        let player = Player::connect_new(sink.mixer());
        player.pause();
        player.append(SquareWave::new(PITCH_HZ).amplify(VOLUME));
        Ok(Self {
            _sink: sink,
            player,
        })
    }

    pub fn set_active(&self, active: bool) {
        if active {
            self.player.play();
        } else {
            self.player.pause();
        }
    }
}
