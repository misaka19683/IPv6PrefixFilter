use clap::Parser;
use env_logger::{Builder, Target};
use ipnet::Ipv6Net;
use log::{debug, info, warn};
use IPv6PrefixFilter::{master::*, AppState};

/// Use IPv6PrefixFilter [COMMAND] --help to see the detail help for each subcommand.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "A simple IPv6 Router Advertisement prefix filter using nftables.",
    long_about = "A simple IPv6 Router Advertisement (RA) prefix filter for Linux that uses nftables \
                  to intercept packets and NFQUEUE to process them in userspace.\n\n\
                  EXAMPLES:\n\
                  1. Allow only specific prefix on eth0:\n\
                     IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64\n\n\
                  2. Block a specific prefix on eth0 (Blacklist mode):\n\
                     IPv6PrefixFilter -i eth0 -b -p 2001:db8:bad::/48\n\n\
                  3. Allow multiple prefixes on eth0:\n\
                     IPv6PrefixFilter -i eth0 -p 2001:db8:1::/64 -p 2001:db8:2::/64\n\n\
                  4. Clear rules and exit:\n\
                     IPv6PrefixFilter --clear"
)]
pub struct Args {
    /// IPv6 prefixes to allow (default) or block (if -b is set).
    #[arg(short = 'p', long = "prefix", value_parser = clap::value_parser!(Ipv6Net))]
    prefixes: Vec<Ipv6Net>,

    /// Network interface to filter on (e.g., eth0).
    #[arg(short = 'i', long)]
    interface: Option<String>,

    /// Enable blacklist mode: prefixes specified with `-p` will be BLOCKED.
    #[arg(short = 'b', long)]
    blacklist: bool,

    /// Clear the nftables rules set by the program and exit.
    #[arg(short = 'c', long)]
    clear: bool,

    /// Disable automatic setup of nftables rules.
    #[arg(long = "no-nft")]
    no_nft: bool,

    /// Verbosity level. Use -v for info, -vv for debug.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();

    // Initialize logging
    match args.verbose {
        0 => env_logger::init(),
        1 => {
            Builder::new().filter_level(log::LevelFilter::Info).target(Target::Stdout).init();
            info!("Logging level set to INFO.");
        }
        _ => {
            Builder::new().filter_level(log::LevelFilter::Debug).target(Target::Stdout).init();
            debug!("Logging level set to DEBUG.");
        }
    };

    if args.clear {
        info!("Clearing nftables rules...");
        if let Err(e) = delete_nftables() {
            eprintln!("Error clearing nftables: {}", e); // 清除 nftables 规则时出错
            std::process::exit(1);
        }
        println!("Nftables rules cleared successfully."); // Nftables 规则已成功清除
        return;
    }
    let mut state = AppState {
        prefixes: args.prefixes,
        blacklist_mode: args.blacklist,
        ..Default::default()
    };

    if let Some(iface_name) = args.interface {
        let interfaces = pnet::datalink::interfaces();
        state.interface = interfaces.into_iter().find(|i| i.name == iface_name);
        if state.interface.is_none() {
            eprintln!("Interface '{}' not found.", iface_name); // 未找到接口
            std::process::exit(1);
        }
    }

    if state.prefixes.is_empty() {
        warn!("No prefixes specified. All Router Advertisements will be ACCEPTED by default.");
    } else {
        for prefix in &state.prefixes {
            info!("{} prefix: {}", if state.blacklist_mode { "Blocking" } else { "Allowing" }, prefix);
        }
    }

    if args.no_nft {
        warn!("Automatic nftables configuration disabled. Please set up rules manually.");
    } else {
        info!("Setting up nftables rules...");
        setup_nftables(&state).expect("Failed to set up nftables");
    }

    info!("Starting RA filter (NFQUEUE listener)...");
    process_queue(state);
    
    // Cleanup on normal exit (though process_queue currently has a loop)
    let _ = delete_nftables();
}
