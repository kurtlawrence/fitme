use clap::Parser;

fn main() -> miette::Result<()> {
    fitme::App::parse().run()
}
