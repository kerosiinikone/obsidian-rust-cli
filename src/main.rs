use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    example: String,
}

fn main() {
    let args = Args::parse();

    eprintln!("{:?}", args.example)
}
