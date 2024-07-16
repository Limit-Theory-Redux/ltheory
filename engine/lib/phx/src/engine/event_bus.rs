use std::collections::{BinaryHeap, HashMap};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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
pub struct Event {
    pub name: String,
    pub priority: i16,
    pub update_pass: UpdatePass,
    // pub callback: fn(Vec<EventPayload>),
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
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            update_pass_map: HashMap::new(),
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
        // callback: fn(Vec<EventPayload>),
        // payloads: Vec<EventPayload>,
    ) {
        let priority = priority.unwrap_or(0);

        let event = Event {
            name: event_name.clone(),
            priority,
            update_pass: update_pass.clone(),
            // callback,
            // payloads,
        };

        self.events.insert(event_name.clone(), event);
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

    pub fn unregister(&mut self, event_name: &str) {
        if let Some(event) = self.events.remove(event_name) {
            if let Some(event_heap) = self.update_pass_map.get_mut(&event.update_pass) {
                event_heap.retain(|e| e.name != event_name);
                println!("Unregistered event: {}", event.name.clone());
            }
        }
    }

    pub fn dispatch(&self, update_pass: UpdatePass, engine: &Engine) {
        //println!("Dispatching for {:?}", update_pass);
        // Print the whole map to verify its state before dispatch
        //self.print_update_pass_map();

        if let Some(event_heap) = self.update_pass_map.get(&update_pass) {
            //println!("Found {} events for {:?}", event_heap.len(), update_pass);
            let events: Vec<_> = event_heap.clone().into_sorted_vec();
            for event_item in events {
                if let Some(event) = self.events.get(&event_item.name) {
                    // (event.callback)(event.payloads.clone());
                    //println!("Dispatched event: {}", event.name.clone());
                    engine.call_lua_func("EventTest")
                } else {
                    panic!("Event not found: {}", event_item.name);
                }
            }
        } else {
            println!("No events found for {:?}", self.update_pass_map.len());
        }
    }

    pub fn dispatch_all(&self, engine: &Engine) {
        for update_pass in UpdatePass::iter() {
            self.dispatch(update_pass, &engine);
        }
    }

    pub fn print_update_pass_map(&self) {
        println!("Current state of update_pass_map:");
        for (update_pass, event_heap) in &self.update_pass_map {
            println!("{:?}", update_pass);
            let events: Vec<_> = event_heap.clone().into_sorted_vec();
            for event_item in events {
                if let Some(event) = self.events.get(&event_item.name) {
                    println!("  {:?}", event_item);
                }
            }
        }
    }
}

// Example usage of the EventBus
// fn example_event_callback(payloads: Vec<String>) {
//     for payload in payloads {
//         println!("Event triggered with payload: {}", payload);
//     }
// }
//
// fn main() {
//     let mut event_bus = EventBus::new();
//
//     event_bus.register(
//         "Event1".to_string(),
//         Some(1),
//         UpdatePass::PreSim,
//         example_event_callback,
//         vec!["Event1".to_string(), "Event1.1".to_string()],
//     );
//     event_bus.register(
//         "Event2".to_string(),
//         Some(2),
//         UpdatePass::PreSim,
//         example_event_callback,
//         vec!["Event2".to_string()],
//     );
//     event_bus.register(
//         "Event3".to_string(),
//         None,
//         UpdatePass::PostFrame,
//         example_event_callback,
//         vec!["Event3".to_string()],
//     );
//     event_bus.register(
//         "Event4".to_string(),
//         Some(1),
//         UpdatePass::PostSim,
//         example_event_callback,
//         vec!["Event4".to_string()],
//     );
//     event_bus.register(
//         "Event5".to_string(),
//         Some(5),
//         UpdatePass::PostSim,
//         example_event_callback,
//         vec!["Event5".to_string()],
//     );
//     event_bus.register(
//         "Event6".to_string(),
//         Some(2),
//         UpdatePass::PostSim,
//         example_event_callback,
//         vec!["Event6".to_string()],
//     );
//
//     // Dispatch events for PreSim
//     println!("Dispatching PreSim events:");
//     event_bus.dispatch(UpdatePass::PreSim);
//
//     // Unregister an event
//     event_bus.unregister("Event1");
//
//     // Dispatch events for PreSim again
//     println!("Dispatching PreSim events after unregistering Event1:");
//     event_bus.dispatch(UpdatePass::PreSim);
//
//     // Dispatch all events in the correct order of update passes
//     println!("Dispatching all events:");
//     event_bus.dispatch_all();
// }
