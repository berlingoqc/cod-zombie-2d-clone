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

    #[structopt(short, long, default_value = "maps/map_iso/iso_map.asset.ron")]
    pub map: String,

    #[structopt(short, long, default_value = "game/easy.level.ron")]
    pub level: String,


    // Temporary two for testing multiplayer
    
    #[structopt(short, long, default_value = "")]
    pub remote_host: String,

    #[structopt(short, long, default_value = "")]
    pub index: u32,
    
}

impl Opts {
    pub fn get() -> Opts {
        return Opts::from_args();
    }
}

