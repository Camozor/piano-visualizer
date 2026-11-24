use anyhow::Result;
use crossterm::{ExecutableCommand, cursor, terminal};
use piano_visualizer::visualizer::Visualizer;

fn main() -> Result<()> {
    let mut visualizer = Visualizer::new();
    let _ = visualizer.run_loop();

    let _ = visualizer.drawer.stdout.execute(cursor::Show);
    let _ = visualizer
        .drawer
        .stdout
        .execute(terminal::LeaveAlternateScreen);

    Ok(())
}
