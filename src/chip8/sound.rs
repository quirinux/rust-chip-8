use rodio::Sink;

pub struct Buzzer {
    sink: rodio::Sink,
}

impl Buzzer {

    fn initialize(&mut self) {
        // Add a dummy source of the sake of the example.
        let source = rodio::source::SineWave::new(440);
        self.sink.append(source);
        self.stop();
    }

    pub fn start(&mut self) {
        if self.sink.empty() {
            self.initialize();
        }
        self.sink.play();
    }

    pub fn stop(&mut self) {
        self.sink.pause();
    }
}

pub fn new() -> Buzzer {
    let device = rodio::default_output_device().unwrap();
    
    Buzzer {
        sink: Sink::new(&device),
    }
}

