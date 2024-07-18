use mlua::{Function, Lua, Table};
use std::{
    borrow::BorrowMut,
    cell::Ref,
    collections::{hash_map::Entry, BinaryHeap, HashMap},
    sync::atomic::{AtomicU32, Ordering},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::trace;

use internal::ConvertIntoString;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum UpdatePass {
    // Before physics update
    PreSim,
    // Physics update
    Sim,
    // After physics update
    PostSim,
    // Before frame render
    PreFrame,
    // Frame render
    Frame,
    // After frame render
    PostFrame,
    // Frame interpolation
    FrameInterpolation,
    // Before input handling
    PreInput,
    // Input handling
    Input,
    // After input handling
    PostInput,
}

impl Default for UpdatePass {
    fn default() -> Self {
        UpdatePass::PreFrame
    }
}

#[derive(Debug, Clone)]
pub enum EventPayload {
    Text(String),
    Number(i32),
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    tunnel_id: u32,
    entity_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub priority: u16,
    pub update_pass: UpdatePass,
    pub subscribers: Vec<Subscriber>,
    // pub payloads: Vec<EventPayload>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MessageRequest {
    priority: u16,
    name: String,
    stay_alive: bool,
    completed: bool,
}

impl Ord for MessageRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for MessageRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct EventBus {
    events: HashMap<String, Event>,
    update_pass_map: HashMap<UpdatePass, BinaryHeap<MessageRequest>>,
    next_tunnel_id: AtomicU32,
    max_priority_locked: bool,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            next_tunnel_id: AtomicU32::new(0),
            update_pass_map: HashMap::new(),
            events: HashMap::new(),
            max_priority_locked: false,
        }
    }

    pub fn lock_max_priority(&mut self) {
        self.max_priority_locked = true;
    }

    pub fn dispatch(&mut self, update_pass: UpdatePass, lua: &mut Ref<Lua>) {
        //* Can we handle the lua state differently? */
        //* Maybe also with future potential multiple lua states in mind */
        let globals = lua.globals();
        let event_tunnels: Table = globals.get("EventTunnels").expect("Unknown table");

        if let Some(message_heap) = self.update_pass_map.get_mut(&update_pass) {
            let mut message_requests: Vec<_> = message_heap.drain().collect();

            for message_request in message_requests.iter_mut() {
                if let Some(event) = self.events.get(&message_request.name) {
                    for subscriber in event.subscribers.iter() {
                        let id = subscriber.tunnel_id;

                        if !message_request.completed && !message_request.stay_alive {
                            println!(
                                "Dispatched event {} through tunnel with id {}",
                                message_request.name, id
                            )
                        }

                        // todo: only dispatch to subscriber with set entity_id if defined
                        let tunnel_func: Function = event_tunnels
                            .get(id)
                            .expect(&format!("Unknown tunnel with id: {}", id));

                        if let Err(e) = tunnel_func.call::<_, ()>(()) {
                            trace!("{}", e);
                        } else {
                            message_request.completed = true; // Mark as completed
                        }
                    }
                } else {
                    panic!("Event not found: {}", message_request.name);
                }
            }

            for message_request in message_requests {
                if message_request.stay_alive || !message_request.completed {
                    message_heap.push(message_request);
                }
            }
        }
    }

    pub fn dispatch_all(&mut self, mut lua: Ref<Lua>) {
        for update_pass in UpdatePass::iter() {
            self.dispatch(update_pass, lua.borrow_mut());
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl EventBus {
    pub fn register(
        &mut self,
        event_name: String,
        priority: Option<u16>,
        update_pass: UpdatePass, //* how do i make the update pass enum work nicely for the lua side? Since the enum is a lua number on lua side.*/
        with_update_pass_message: bool,
    ) {
        // default priority 0
        let priority = priority.unwrap_or(0);

        if self.max_priority_locked {
            if priority == u16::MAX {
                panic!("Trying to register event at locked priority");
            }
        }

        let event = Event {
            name: event_name.clone(),
            priority,
            update_pass: update_pass,
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

                if with_update_pass_message {
                    let event_item = MessageRequest {
                        priority,
                        name: event_name.clone(),
                        stay_alive: with_update_pass_message,
                        completed: false,
                    };

                    self.update_pass_map
                        .entry(update_pass)
                        .or_insert_with(|| {
                            println!("Inserting new BinaryHeap for {:?}", update_pass);
                            BinaryHeap::new()
                        })
                        .push(event_item.clone());

                    // Verify the message_heap immediately after insertion
                    if let Some(message_heap) = self.update_pass_map.get(&update_pass) {
                        println!(
                            "Event heap size after registration for {:?}: {}",
                            update_pass,
                            message_heap.len()
                        );
                        for message_request in message_heap.iter() {
                            if let Some(event) = self.events.get(&message_request.name) {
                                println!("Registered event: {}", event.name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn unregister(&mut self, event_name: &str) {
        if let Some(event) = self.events.remove(event_name) {
            if let Some(message_heap) = self.update_pass_map.get_mut(&event.update_pass) {
                message_heap.retain(|e| e.name != event_name);
                println!("Unregistered event: {}", event.name.clone());
            }
        }
    }

    /// @overload fun(self: table, eventName: string, ctxTable: table|nil, callbackFunc: function): integer
    pub fn subscribe(&mut self, event_name: String, entity_id: Option<u32>) -> u32 {
        //* should return handler instead of u32 */
        //* how do i handle Options via ffi? It requires a uint16 const pointer */
        let tunnel_id = self.next_tunnel_id.fetch_add(1, Ordering::SeqCst);

        if let Some(event) = self.events.get_mut(&event_name) {
            let subscriber = Subscriber {
                tunnel_id,
                entity_id,
            };
            event.subscribers.push(subscriber);
        }

        println!(
            "Subscribed to event '{}' with tunnel_id {}",
            event_name, tunnel_id
        );
        tunnel_id
    }

    pub fn unsubscribe(&mut self, tunnel_id: u32) {
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

    pub fn send(
        &mut self,
        event_name: String,
        entity_id: u32,
        //* Add payload here later */
    ) {
        if let Some(event) = self.events.get(&event_name) {
            //for subscriber in &event.subscribers {
            //    if subscriber.entity_id == Some(entity_id) {
            let message_request = MessageRequest {
                priority: event.priority,
                name: event_name.clone(),
                stay_alive: false, // item will be consumed on dispatch
                completed: false,
            };

            self.update_pass_map
                .entry(event.update_pass)
                .or_insert_with(|| {
                    println!("Inserting new BinaryHeap for {:?}", event.update_pass);
                    BinaryHeap::new()
                })
                .push(message_request);

            println!("Created send message request: {}", event.name.clone());
            //    }
            //}
        } else {
            panic!("Event not found: {}", event_name);
        }
    }

    pub fn print_update_pass_map(&self) {
        println!("Current state of update_pass_map:");
        for (update_pass, event_heap) in self.update_pass_map.iter() {
            println!("{:?}", update_pass);
            for message_request in event_heap.iter() {
                if let Some(_event) = self.events.get(&message_request.name) {
                    println!(" - {:?}", message_request);
                }
            }
        }
    }
}
