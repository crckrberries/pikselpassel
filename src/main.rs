use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;
use rand::*;
use std::io::{self};
mod cmd;
mod frame;
mod sender;
mod text;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    /// the command to run
    cmd: Commands,
    /// hostname and port (example: pixelflut.organs.trade:1234)
    host: String,
    /// offset from the top left of the canvas, in pixels
    offset: String,

    /// whether to shuffle the pixel order or not
    #[arg(long, short = 's', default_value_t = false)]
    shuffle: bool,
    /// whether to loop the drawing cycle or not
    #[arg(long, short = 'l', default_value_t = false)]
    looping: bool,
}
#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// send an image to the pixelflut canvas
    Img {
        path: String,
        size: String,
    },

    /// wipe the pixelflut canvas
    Wipe {
        size: String,
    },

    /// render and send text to the pixelflut canvas
    Text {
        text: String,
        size: f32,
    },

    Gif {
        path: String,
        size: String,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let host: String = cli.host; // replace with ip and port you need

    let mut off = cli.offset.split('x');
    let xoff: u32 = off.next().unwrap().parse().unwrap();
    let yoff: u32 = off.next().unwrap().parse().unwrap();

    let looping: bool = cli.looping; // whether to loop the draw cycle or not
    let shuffle: bool = cli.shuffle; // whether to randomize the sequence of the commands, creating a dithering effect

    let mut frames: Vec<frame::Frame> = vec![];
    match cli.cmd {
        Commands::Img { path, size } => {
            let mut size = size.split('x');
            let sizex: u32 = size.next().unwrap().parse().unwrap();
            let sizey: u32 = size.next().unwrap().parse().unwrap();
            let img = cmd::read_image(path, sizex, sizey);
            let cmds = cmd::process_image(&img, xoff, yoff); // processes image, generating commands
            frames.push(frame::Frame { commands: cmds, delay: 0})
        }

        Commands::Wipe { size } => {
            let mut size = size.split('x');
            let sizex: u32 = size.next().unwrap().parse().unwrap();
            let sizey: u32 = size.next().unwrap().parse().unwrap();
            let cmds = cmd::wipe(sizex, sizey); // wipes screen
            frames.push(frame::Frame { commands: cmds, delay: 0})
        }

        Commands::Text { text, size } => {
            let cmds =
                cmd::process_image(&text::render_text(text, size, (255, 255, 255)), xoff, yoff);
            frames.push(frame::Frame { commands: cmds, delay: 0})
            
        }

        Commands::Gif { path, size } => {
            let mut size = size.split('x');
            let sizex: u32 = size.next().unwrap().parse().unwrap();
            let sizey: u32 = size.next().unwrap().parse().unwrap();
            let img = cmd::read_gif(path);
            frames = cmd::process_gif(img, sizex, sizey);
        }
    };

    if shuffle {
        for mut frame in frames.clone() {
            frame.commands.shuffle(&mut thread_rng());
        }
    }

    match looping {
        true => sender::sendloop(frames, &host),

        false => sender::send(frames, &host),
    }

    // writer.flush().unwrap();

    Ok(())
}
