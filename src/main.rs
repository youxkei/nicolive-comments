/// A CUI tool for fetching comments from nicolive.
#[derive(structopt::StructOpt, Debug)]
struct Args {
    /// A live URL whose comments will be fatched
    #[structopt(name = "URL", parse(try_from_str))]
    url: url::Url,
}

#[paw::main]
fn main(args: Args) {
    println!("{:?}", args);
}
