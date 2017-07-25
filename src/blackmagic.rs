use clap::ArgMatches;
use config::Config;
use Result;

pub fn blackmagic_scan(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches) -> Result<&'static str> {
    let blackmagic_mode = if let Some(blackmagic_mode) = cfg.blackmagic_mode() {
        blackmagic_mode
    } else if let Some(blackmagic_mode) = cmd_args.value_of("blackmagic-mode") {
        blackmagic_mode
    } else {
        "swd"
    };
    match blackmagic_mode {
        "jtag" => Ok("monitor jtag_scan"),
        "swd" => Ok("monitor swdp_scan"),
        _ => bail!("Unknown blackmagic-mode {}", blackmagic_mode),
    }
}