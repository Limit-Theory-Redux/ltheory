use mlua::{Function, Table};
use std::{
    collections::{hash_map::Entry, BinaryHeap, HashMap},
    sync::atomic::{AtomicI32, Ordering},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::trace;

use internal::ConvertIntoString;

use super::Engine;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum UpdatePass {
    PreSim,             // Before physics update
    Sim,                // Physics update
    PostSim,            // After physics update
    PreFrame,           // Before frame render
    Frame,              // Frame render
    PostFrame,          // After frame render
    FrameInterpolation, // Frame interpolation
    PreInput,           // Before input handling
    Input,              // Input handling
    PostInput,          // After input handling
}

#[derive(Debug, Clone)]
pub enum EventPayload {
    Text(String),
    Number(i32),
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    tunnel_id: i32,
    entity_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub priority: i16,
    pub update_pass: UpdatePass,
    pub subscribers: Vec<Subscriber>,
    // pub payloads: Vec<EventPayload>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct EventItem {
    priority: i16,
    name: String,
}

impl Ord for EventItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for EventItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct EventBus {
    events: HashMap<String, Event>,
    update_pass_map: HashMap<UpdatePass, BinaryHeap<EventItem>>,
    next_tunnel_id: AtomicI32,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            next_tunnel_id: AtomicI32::new(0),
            update_pass_map: HashMap::new(),
            events: HashMap::new(),
        }
    }

    // todo event action queue & manual dispatching
    pub fn dispatch(&self, update_pass: UpdatePass, engine: &Engine) {
        let lua = engine.lua.as_ref();
        let globals = lua.globals();
        let event_tunnels: Table = globals.get("EventTunnels").expect("Unknown table");

        if let Some(event_heap) = self.update_pass_map.get(&update_pass) {
            let mut events: Vec<_> = event_heap.iter().collect();
            events.sort_by(|a, b| a.priority.cmp(&b.priority)); // Sort events without cloning

            for event_item in events {
                if let Some(event) = self.events.get(&event_item.name) {
                    for subscriber in &event.subscribers {
                        let id = subscriber.tunnel_id;

                        let tunnel_func: Function = event_tunnels
                            .get(id)
                            .expect(&format!("Unknown tunnel with id: {}", id));
                        if let Err(e) = tunnel_func.call::<_, ()>(()) {
                            trace!("{}", e);
                        }
                    }
                } else {
                    panic!("Event not found: {}", event_item.name);
                }
            }
        }
    }

    pub fn dispatch_all(&self, engine: &Engine) {
        for update_pass in UpdatePass::iter() {
            self.dispatch(update_pass, &engine);
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl EventBus {
    pub fn register(
        &mut self,
        event_name: String,
        priority: Option<i16>,
        update_pass: UpdatePass,
        // payloads: Vec<EventPayload>,
    ) {
        let priority = priority.unwrap_or(0);

        let event = Event {
            name: event_name.clone(),
            priority,
            update_pass: update_pass.clone(),
            subscribers: vec![],
            // payloads,
        };

        match self.events.entry(event_name.clone()) {
            Entry::Occupied(_) => {
                println!(
                    "Warning: You are trying to register an Event '{}' that already exists - Aborting!",
                    event_name
                );
            }
            Entry::Vacant(entry) => {
                entry.insert(event);

                let event_item = EventItem {
                    priority,
                    name: event_name.clone(),
                };

                self.update_pass_map
                    .entry(update_pass)
                    .or_insert_with(|| {
                        println!("Inserting new BinaryHeap for {:?}", update_pass);
                        BinaryHeap::new()
                    })
                    .push(event_item.clone());

                // Verify the event_heap immediately after insertion
                if let Some(event_heap) = self.update_pass_map.get(&update_pass) {
                    let events: Vec<_> = event_heap.clone().into_sorted_vec();
                    println!(
                        "Event heap size after registration for {:?}: {}",
                        update_pass,
                        event_heap.len()
                    );
                    for event_item in events {
                        if let Some(event) = self.events.get(&event_item.name) {
                            println!("Registered event: {}", event.name.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn unregister(&mut self, event_name: &str) {
        if let Some(event) = self.events.remove(event_name) {
            if let Some(event_heap) = self.update_pass_map.get_mut(&event.update_pass) {
                event_heap.retain(|e| e.name != event_name);
                println!("Unregistered event: {}", event.name.clone());
            }
        }
    }

    pub fn subscribe(&mut self, event_name: String, entity_id: Option<i32>) -> i32 {
        let tunnel_id = self.next_tunnel_id.fetch_add(1, Ordering::SeqCst);

        for event_heap in self.update_pass_map.values() {
            for event_item in event_heap.iter() {
                if let Some(event) = self.events.get_mut(&event_item.name) {
                    if event_item.name == event_name {
                        let subscriber = Subscriber {
                            tunnel_id,
                            entity_id,
                        };
                        event.subscribers.push(subscriber);
                    }
                }
            }
        }

        println!(
            "Subscribed to event '{}' with tunnel_id {}",
            event_name, tunnel_id
        );
        tunnel_id
    }

    pub fn unsubscribe(&mut self, tunnel_id: i32) {
        for event in self.events.values_mut() {
            event
                .subscribers
                .retain(|subscriber| subscriber.tunnel_id != tunnel_id);
        }

        println!(
            "Unsubscribed from event and closed tunnel with id: {}",
            tunnel_id
        );
    }

    pub fn send(&self, event_name: String, entity_id: i32) {
        if let Some(event) = self.events.get(&event_name) {
            // dispatch event with entity_id payload
            for subscriber in &event.subscribers {
                if subscriber.entity_id == Some(entity_id) {
                    let id = subscriber.tunnel_id;
                    //* instead of calling function here, add to event queue or to next tick and then consume this event */

                    // let tunnel_func: Function = event_tunnels
                    //     .get(id)
                    //     .expect(&format!("Unknown tunnel with id: {}", id));
                    // if let Err(e) = tunnel_func.call::<_, ()>(()) {
                    //     trace!("{}", e);
                    // }
                }
            }
        }
    }

    pub fn print_update_pass_map(&self) {
        println!("Current state of update_pass_map:");
        for (update_pass, event_heap) in &self.update_pass_map {
            println!("{:?}", update_pass);
            let events: Vec<_> = event_heap.clone().into_sorted_vec();
            for event_item in events {
                if let Some(_event) = self.events.get(&event_item.name) {
                    println!("  {:?}", event_item);
                }
            }
        }
    }
}
