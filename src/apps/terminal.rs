use std::rc::Rc;
use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::consts::{ANSI_RED, ANSI_RESET};
use crate::services::Shell;
use crate::sys::{self, Process, WindowInstance, WindowInstanceProps};

pub fn new_terminal_instance() -> Process {
    let mut process = Process::new("terminal");
    let pid = process.id;
    process.add_window(WindowInstance::new(WindowInstanceProps {
        title: "terminal".to_owned(),
        size: ScreenCoordinates::Absolute { x: 350, y: 250 },
        on_close: Some(Rc::new(move |_| {
            sys::kill_process(pid);
        })),
        ..Default::default()
    }, move |_| rsx! {
        WindowTerminal { pid }
    }));
    
    process
}

#[component]
fn WindowTerminal(pid: u32) -> Element {
    let mut shell = use_signal(Shell::default);
    let container_id = use_memo(move || format!("terminal-container-{}", pid));
    
    use_effect(move || {
        let js = format!(r#"
            const term = new Terminal({{
                fontFamily: 'Tamzen8x16',
                fontSize: 16,
                theme: {{ background: '#131313' }}
            }});
            
            const fitAddon = new FitAddon.FitAddon();
            term.loadAddon(fitAddon);
            
            const container = document.getElementById('{container_id}');
            if (container) {{
                term.open(container);
                const resizeObserver = new ResizeObserver(() => {{
                    if (container.clientWidth > 0 && container.clientHeight > 0) {{
                        fitAddon.fit();
                    }}
                }});
                
                resizeObserver.observe(container);
                fitAddon.fit();
                
                term.write('{ANSI_RED}${ANSI_RESET} ');
                
                let currentLine = "";
            
                term.onData(e => {{
                    if (e === '\r') {{
                        dioxus.send(currentLine);
                        currentLine = "";
                    }} else if (e === '\u007f') {{ // Backspace
                        if (currentLine.length > 0) {{
                            currentLine = currentLine.slice(0, -1);
                            term.write('\b \b');
                        }}
                    }} else {{
                        currentLine += e;
                        term.write(e);
                    }}
                }});
                
                // Listen for output from Rust
                while (true) {{
                    let msg = await dioxus.recv();
                    term.write('\r\n' + msg + '{ANSI_RED}${ANSI_RESET} ');
                }}
            }}
        "#);
    
        let mut eval = document::eval(&js);
        spawn(async move {
            while let Ok(input) = eval.recv::<String>().await {
                let output = shell.with_mut(|s| s.handle_input(&input));
                let _ = eval.send(output);
            }
        });
    });
    
    rsx! {
        div { 
            id: "{container_id}",
            style: "width: 100%; height: 100%; background: #131313;"
        }
    }
}