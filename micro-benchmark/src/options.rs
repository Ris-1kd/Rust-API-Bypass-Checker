use clap::Parser;
use clap::ValueEnum;

#[derive(Parser,Debug)]
#[command(
    name ="Function Nmae Displayer", 
    version = "1.0", 
    about = "A compiler plugin to show the APIs' fully names.",
)]
pub struct Cli{
    /// The input file
    pub input_file: String
}
