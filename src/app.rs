use crossterm::event::KeyCode;
use std::fs;
use std::path::Path;
use crate::utils::Agency;

#[derive(Debug)]
pub struct App {
    pub items: Vec<String>,
    pub selected: usize,
    pub should_quit: bool,
    pub agencies: Vec<Agency>,
}

impl App {
    pub fn new() -> Self {
        Self {
            items: vec![],
            selected: 0,
            should_quit: false,
            agencies: vec![],
        },
        
    }

    pub fn readconfig(&mut self) {
        let config_dir = Path::new("./config");
        if !config_dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(agency) = serde_json::from_str::<Agency>(&content) {
                            for node in &agency.node {
                                self.items.push(format!("[{}] {}:{} - {}", 
                                    node.protocol, node.address, node.port, node.name));
                            }
                            self.agencies.push(agency);
                        }
                    }
                }
            }
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Up => {
                if(self.items.len()==0)
                {
                    self.selected=0;
                }
                else if(self.selected>0){
                    self.selected -= 1;
                }else{
                    self.selected=self.items.len()-1;
                }
            }
            KeyCode::Down => {
                if ( self.items.len()==0 )
                {
                    self.selected=0;
                }else{
                self.selected =(self.selected+1)%self.items.len();
                }
            }
            KeyCode::Enter=>{
                 if ( self.items.len()==0 ) return;

            }
            _ => {}
        }
    }
}