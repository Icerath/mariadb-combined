use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

pub fn init() {
    let mut config = ConfigBuilder::new();
    let _ = config.set_time_offset_to_local();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            config.build(),
            File::create("log.log").unwrap(),
        ),
    ])
    .unwrap();
}
