use structopt::StructOpt;

#[derive(StructOpt, Debug, Default, Clone)]
#[structopt(name = "basic")]
pub struct Opts {
    #[structopt(short, long)]
    pub benchmark_mode: bool,

    #[structopt(short, long)]

    #[structopt(short, long, default_value = "8")]
    pub key_frame_interval: usize,

    #[structopt(short, long, default_value = "60")]
    pub fps: usize,

    #[structopt(short, long, default_value = "")]
    pub host: String,

    #[structopt(short, long, default_value = "0")]
    pub port: i32,

    #[structopt(short, long, default_value = "level_1.custom")]
    pub map: String,

    #[structopt(short, long, default_value = "level_1.level")]
    pub level: String
}

impl Opts {
    pub fn get() -> Opts {
        return Opts::from_args();
    }
}

