use std::net::IpAddr;
use crate::Error;

use std::str::FromStr;

use std::process::exit;

#[derive(Debug)]
pub struct Conf {
    pub addr: IpAddr,
    pub port: u16,
}

const USAGE: &str = "Usage: rst [-aphV]";
const ERR_MESSAGE: &str = "Try the `-h` flag for more info.";
const HELP_MESSAGE: &str =
"\x1b[1mDESCRIPTION\x1b[0m
    \x1b[4mrst\x1b[0m is a framework for creating static sites in pure rust.
    
\x1b[1mOPTIONS\x1b[0m
    -V  print current program version
    -h  print this message

    -a ADDRESS
        Bind on the specified ADDRESS

    -p PORT
        Bind to the specified PORT
";

impl Default for Conf {
    fn default() -> Self {
        Self {
            addr: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 8080,
        }
    }
}

impl Conf {
    pub fn parse(args: Vec<String>) -> Result<Conf, Error> {
        let mut out = Conf::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            if let Some(arg) = arg.strip_prefix('-') {
                for c in arg.chars() {
                    match c {
                        'h' => {
                            println!("{}\n\n{}", USAGE, HELP_MESSAGE);
                            exit(0);
                        },

                        'V' => {
                            println!("{}", env!("CARGO_PKG_VERSION"));
                            exit(0);
                        },

                        /* address */
                        'a' => out.addr = match args.next() {
                            Some(a) => IpAddr::from_str(&a)?,
                            None => return Err("Missing argument after `-a`".into()),
                        },

                        /* port */
                        'p' => out.port = match args.next() {
                            Some(a) => a.parse::<u16>()?,
                            None => return Err("Missing argument after `-p`".into()),
                        },

                        _ => {
                            println!("{}\n{}", USAGE, ERR_MESSAGE);
                            exit(1);
                        },
                    }
                } continue;
            }
        } Ok(out)
    }
}
