use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version = "0.1.0", about = "A cli for BUAA")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
    Evaluation(Evaluation),
    #[command(about = "Wifi Login & Logout")]
    Wifi(Wifi),
}

#[derive(Parser, Debug)]
pub struct Boya {
    #[clap(subcommand)]
    pub command: BoyaSub,
}

#[derive(Debug, Subcommand)]
pub enum BoyaSub {
    /// Refresh token
    Login,
    /// Query courses and select a course by ID
    Query {
        #[arg(short, long)]
        /// By default, only optional courses are displayed
        all: bool,
    },
    /// Selete by ID immediately. No guarantee for valid token
    Select { id: u32 },
    /// Drop by ID
    Drop { id: u32 },
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
    /// Refresh token
    Login,
    /// Auto checkin for today
    Auto,
    /// Query term courses with Term ID or course schedule with Course ID
    Query {
        /// Term ID or Course ID
        id: Option<String>,
    },
    /// Checkin with Course ID, or Schedule ID
    Checkin {
        /// Course ID, or Schedule ID
        id: String,
        #[arg(short, long)]
        /// Delay time. eg. '0800' means 8:00 am.
        time: Option<String>,
    },
}

#[derive(Parser, Debug)]
pub struct Evaluation {
    #[clap(subcommand)]
    pub command: EvaluationSub,
}

#[derive(Debug, Subcommand)]
pub enum EvaluationSub {
    /// Warning!!! No Tested!!!
    Auto,
    List,
    Fill,
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
