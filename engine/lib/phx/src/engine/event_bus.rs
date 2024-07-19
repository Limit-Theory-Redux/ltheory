use std::{
    collections::{hash_map::Entry, BTreeSet, BinaryHeap, HashMap},
    sync::atomic::{AtomicU32, Ordering},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Subscriber {
    id: u32,
    tunnel_id: u32,
    entity_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub priority: u16,
    pub update_pass: UpdatePass,
    pub subscribers: Vec<Subscriber>,
    pub processed_subscribers: Vec<usize>,
}

impl Event {
    fn get_next_subscriber(&mut self) -> Option<&Subscriber> {
        for i in 0..self.subscribers.len() {
            if !self.processed_subscribers.contains(&i) {
                self.processed_subscribers.push(i);
                return self.subscribers.get(i);
            }
        }
        None
    }

    fn reset_processed_subscribers(&mut self) {
        self.processed_subscribers.clear();
    }
}

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    SomeType,
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub update_pass: UpdatePass,
    pub event_type: EventType,
    pub tunnel_id: u32,
}

#[luajit_ffi_gen::luajit_ffi]
impl EventData {
    pub fn get_update_pass(&self) -> UpdatePass {
        self.update_pass
    }

    pub fn get_tunnel_id(&self) -> u32 {
        self.tunnel_id
    }

    pub fn get_event_type(&self) -> EventType {
        self.event_type
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MessageRequest {
    id: u32,
    priority: u16,
    event_name: String,
    stay_alive: bool,
}

impl Ord for MessageRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.event_name.cmp(&other.event_name))
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
    cached_stay_alive_requests: Vec<MessageRequest>,
    next_subscriber_id: AtomicU32,
    next_message_id: AtomicU32,
    next_tunnel_id: AtomicU32,
    message_ids: BTreeSet<u32>,
    max_priority_locked: bool,
    current_update_pass: Option<UpdatePass>,
    current_message_request: Option<MessageRequest>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            update_pass_map: HashMap::new(),
            cached_stay_alive_requests: vec![],
            next_subscriber_id: AtomicU32::new(0),
            next_message_id: AtomicU32::new(0),
            next_tunnel_id: AtomicU32::new(0),
            message_ids: BTreeSet::new(),
            max_priority_locked: false,
            current_update_pass: Some(UpdatePass::PreSim),
            current_message_request: None,
        }
    }

    pub fn lock_max_priority(&mut self) {
        self.max_priority_locked = true;
    }

    pub fn add_subscriber(&mut self, event_name: &str, tunnel_id: u32, entity_id: Option<u32>) {
        let subscriber_id = self.next_subscriber_id.fetch_add(1, Ordering::SeqCst);
        let subscriber = Subscriber {
            id: subscriber_id,
            tunnel_id,
            entity_id,
        };

        if let Some(event) = self.events.get_mut(event_name) {
            event.subscribers.push(subscriber);
        } else {
            panic!("error while pushing subscriber");
        }
    }

    fn get_unique_message_id(&mut self) -> u32 {
        if let Some(&id) = self.message_ids.iter().next() {
            self.message_ids.remove(&id);
            id
        } else {
            self.next_message_id.fetch_add(1, Ordering::SeqCst)
        }
    }

    fn release_unique_message_id(&mut self, id: u32) {
        self.message_ids.insert(id);
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
            processed_subscribers: vec![],
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
                    let message_request = MessageRequest {
                        id: self.get_unique_message_id(),
                        priority,
                        event_name: event_name.clone(),
                        stay_alive: with_update_pass_message,
                    };

                    self.update_pass_map
                        .entry(update_pass)
                        .or_insert_with(|| {
                            println!("Inserting new BinaryHeap for {:?}", update_pass);
                            BinaryHeap::new()
                        })
                        .push(message_request);

                    // Verify the message_vec immediately after insertion
                    if let Some(message_heap) = self.update_pass_map.get(&update_pass) {
                        println!(
                            "Heap size after registration for {:?}: {}",
                            update_pass,
                            message_heap.len()
                        );
                        for message_request in message_heap.iter() {
                            if let Some(event) = self.events.get(&message_request.event_name) {
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
                message_heap.retain(|e| e.event_name != event_name);
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
            self.add_subscriber(&event_name, tunnel_id, entity_id)
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
        // Add payload here later
    ) {
        // Temporarily remove the event from the HashMap to avoid borrowing issues
        if let Some(event) = self.events.remove(&event_name) {
            // Find the subscriber with the matching entity_id
            if let Some(subscriber) = event
                .subscribers
                .iter()
                .find(|s| s.entity_id == Some(entity_id))
            {
                let message_request = MessageRequest {
                    id: self.get_unique_message_id(),
                    priority: event.priority,
                    event_name: event_name.clone(),
                    stay_alive: false, // item will be consumed on dispatch
                };

                // Add the message request to the update_pass_map
                self.update_pass_map
                    .entry(event.update_pass)
                    .or_insert_with(BinaryHeap::new)
                    .push(message_request);
            }

            // Insert the event back into the HashMap
            self.events.insert(event_name, event);
        }
    }

    pub fn get_next_event(&mut self) -> Option<&EventData> {
        static mut EVENT_DATA_STORAGE: Option<EventData> = None;

        while let Some(update_pass) = self.current_update_pass {
            if let Some(queue) = self.update_pass_map.get_mut(&update_pass) {
                if let Some(message_request) = queue.peek().cloned() {
                    self.current_message_request = Some(message_request.clone());

                    if let Some(event) = self.events.get_mut(&message_request.event_name) {
                        event.subscribers.sort_by(|a, b| a.id.cmp(&b.id));

                        if let Some(subscriber) = event.get_next_subscriber() {
                            let event_data = EventData {
                                update_pass: update_pass,
                                event_type: EventType::SomeType,
                                tunnel_id: subscriber.tunnel_id,
                            };

                            unsafe {
                                EVENT_DATA_STORAGE = Some(event_data);
                            }

                            return unsafe { EVENT_DATA_STORAGE.as_ref() };
                        } else {
                            // No more subscribers to process, clear queue
                            event.reset_processed_subscribers();
                            queue.clear();

                            if message_request.stay_alive {
                                self.cached_stay_alive_requests
                                    .push(message_request.clone());
                            } else {
                                self.release_unique_message_id(message_request.id);
                            }
                        }
                    }
                }
            }

            let next_update_pass = {
                let mut iter = UpdatePass::iter().skip_while(|&pass| pass != update_pass);
                iter.next();
                iter.next()
            };

            //tracing::info!(
            //    "Finished UpdatePass: {:?} - Next UpdatePass: {:?}",
            //    self.current_update_pass,
            //    next_update_pass
            //);

            self.current_update_pass = next_update_pass;

            // Reinsert stay-alive
            for message_request in self.cached_stay_alive_requests.drain(..) {
                self.update_pass_map
                    .entry(update_pass)
                    .or_insert_with(BinaryHeap::new)
                    .push(message_request);
            }
        }
        self.current_update_pass = Some(UpdatePass::PreSim);
        None
    }

    pub fn print_update_pass_map(&self) {
        println!("Current state of update_pass_map:");

        // Create a sorted vector of UpdatePass keys based on the enum order
        let mut sorted_keys: Vec<_> = UpdatePass::iter().collect();

        for update_pass in sorted_keys {
            if let Some(event_heap) = self.update_pass_map.get(&update_pass) {
                println!("{:?}", update_pass);
                for message_request in event_heap.iter() {
                    if let Some(_event) = self.events.get(&message_request.event_name) {
                        println!(" - {:?}", message_request);
                    }
                }
            }
        }
    }
}
