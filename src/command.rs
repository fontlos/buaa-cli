use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version = "0.3.1", about = "A cli for BUAA")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(short, long, default_value_t = false)]
    pub disable_tls: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Login for SSO")]
    Login {
        #[clap(short, long)]
        username: Option<String>,
        #[clap(short, long)]
        password: Option<String>,
    },
    #[command(about = "Boya Course")]
    Boya(Boya),
    #[command(about = "Smart Classroom")]
    Class(Class),
    #[command(about = "Teacher Evaluation system")]
    Tes(Tes),
    #[command(about = "BUAA-Wifi")]
    Wifi(Wifi),
}

#[derive(Parser, Debug)]
pub struct Boya {
    #[clap(subcommand)]
    pub command: BoyaSub,
}

#[derive(Debug, Subcommand)]
pub enum BoyaSub {
    /// Query courses and select a course by ID
    Query {
        #[arg(short, long)]
        /// By default, only optional courses are displayed
        all: bool,
        /// Page number (Default 1)
        #[arg(short, long, default_value_t = 1)]
        page: u8,
    },
    /// Selete by ID immediately. No guarantee for valid token
    Select { id: u32 },
    /// Drop by ID
    Drop { id: u32 },
    /// Check-in/out by ID, Only for courses that can check-in/out by self
    Check { id: u32 },
    /// Query statistics information, use `-s` to show selected courses
    Status {
        #[arg(short, long)]
        /// By default, only statistic information is displayed
        selected: bool,
    },
}

#[derive(Parser, Debug)]
pub struct Class {
    #[clap(subcommand)]
    pub command: ClassSub,
}

#[derive(Debug, Subcommand)]
pub enum ClassSub {
    /// Auto checkin for today
    Auto,
    /// Query schedules by Date, term courses by Term ID, or course schedules by Course ID
    Query {
        /// Date (format: YYYYMMDD),
        /// Term ID (e.g. `202320242` is 2024 spring term, `202420251` is 2024 autumn term)
        /// or Course ID (from query term courses)
        id: String,
    },
    /// Checkin with Schedule ID or Date (format: YYYYMMDD)
    Checkin {
        /// Schedule ID or Date (format: YYYYMMDD)
        id: String,
    },
}

#[derive(Parser, Debug)]
pub struct Tes {
    #[clap(subcommand)]
    pub command: TesSub,
}

#[derive(Debug, Subcommand)]
pub enum TesSub {
    /// Warning!!! No Tested!!!
    Auto,
    List {
        #[arg(short, long)]
        /// By default, only unfilled forms are displayed
        all: bool,
    },
}

#[derive(Parser, Debug)]
pub struct Wifi {
    #[clap(subcommand)]
    pub command: WifiSub,
}

#[derive(Debug, Subcommand)]
pub enum WifiSub {
    Login,
    Logout,
}
