use webpush_sender::push::proc::RuntimeConfig;

fn main() {
    let cfg = match RuntimeConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("config err class={e:?}");
            std::process::exit(2);
        }
    };
    if let Err(e) = webpush_sender::push::proc::run(cfg) {
        eprintln!("fatal err class={e:?}");
        std::process::exit(1);
    }
}
