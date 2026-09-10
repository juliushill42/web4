use crossterm::{cursor, event::{self, Event, KeyCode, KeyModifiers}, execute, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType}};
use std::io::{stdout, Write};
use tokio::sync::mpsc;

pub async fn run_ui(mut input_tx: mpsc::Sender<String>, mut output_rx: mpsc::Receiver<String>) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    let mut input_buffer = String::new();
    let mut messages: Vec<String> = Vec::new();
    draw_screen(&mut stdout, &messages, &input_buffer)?;

    loop {
        tokio::select! {
            Some(msg) = output_rx.recv() => {
                messages.push(msg);
                if messages.len() > 20 { messages.remove(0); }
                draw_screen(&mut stdout, &messages, &input_buffer)?;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {
                if event::poll(std::time::Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => { 
                                disable_raw_mode()?; 
                                return Ok(()); // Graceful exit, triggers shutdown channel 
                            }
                            KeyCode::Enter => {
                                if !input_buffer.is_empty() {
                                    input_tx.send(input_buffer.clone()).await?;
                                    messages.push(format!("(You): {}", input_buffer)); 
                                    input_buffer.clear();
                                    draw_screen(&mut stdout, &messages, &input_buffer)?;
                                }
                            }
                            KeyCode::Char(c) => { input_buffer.push(c); draw_screen(&mut stdout, &messages, &input_buffer)?; }
                            KeyCode::Backspace => { input_buffer.pop(); draw_screen(&mut stdout, &messages, &input_buffer)?; }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn draw_screen(stdout: &mut std::io::Stdout, messages: &[String], input: &str) -> std::io::Result<()> {
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    println!("WEB4 GHOSTCHAT [SECURE MESH ACTIVE]\r");
    println!("-------------------------------------\r");
    for msg in messages { println!("{}\r", msg); }
    execute!(stdout, cursor::MoveTo(0, 22))?;
    print!("> {}", input);
    stdout.flush()?;
    Ok(())
}
