use std::rc::Rc;
use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
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
    let container_id = use_memo(move || format!("terminal-container-{}", pid));
    
    let mut shell = use_signal(Shell::default);
    
    use_effect(move || {
        let js = format!("
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
            
                // stdin
                term.onData(e => {{
                    dioxus.send(e);
                }});
                
                // stdout
                while (true) {{
                    let msg = await dioxus.recv();
                    term.write(msg);
                }}
            }}
        ");
    
        let mut eval = document::eval(&js);
        spawn(async move {
            while let Ok(input) = eval.recv::<String>().await {
                shell.with_mut(|s| s.handle_stdin(&input));
            }
        });
        
        spawn(async move {
            loop {
                let out = {
                    let rx = shell.with(|s| s.rx.clone());
                    match rx.recv().await {
                        Ok(v) => v,
                        Err(_) => break,
                    }
                };

                let _ = eval.send(out);
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