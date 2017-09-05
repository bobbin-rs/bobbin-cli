use clap::ArgMatches;
use config::Config;
use Result;

pub fn blackmagic_scan(cfg: &Config, args: &ArgMatches, cmd_args: &ArgMatches) -> Result<&'static str> {
    let blackmagic_mode = if let Some(blackmagic_mode) = cfg.blackmagic_mode(cmd_args) {
        blackmagic_mode
    } else {
        String::from("swd")
    };
    match blackmagic_mode.as_ref() {
        "jtag" => Ok("monitor jtag_scan"),
        "swd" => Ok("monitor swdp_scan"),
        _ => bail!("Unknown blackmagic-mode {}", blackmagic_mode),
    }
}