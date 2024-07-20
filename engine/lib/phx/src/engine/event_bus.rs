use std::{
    collections::{hash_map::Entry, BinaryHeap, HashMap, VecDeque},
    sync::atomic::{AtomicU32, Ordering},
};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use internal::ConvertIntoString;
use tracing::{info, warn};

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

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EventPriority {
    Lowest = 0,
    VeryLow = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    Higher = 5,
    VeryHigh = 6,
    Max = 255,
}

impl Ord for EventPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for EventPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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
    pub priority: EventPriority,
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
struct MessageRequestCache {
    update_pass: UpdatePass,
    priority: EventPriority,
    event_name: String,
    stay_alive: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MessageRequest {
    priority: EventPriority,
    event_name: String,
    stay_alive: bool,
}

impl From<MessageRequestCache> for MessageRequest {
    fn from(cache: MessageRequestCache) -> Self {
        MessageRequest {
            priority: cache.priority,
            event_name: cache.event_name,
            stay_alive: cache.stay_alive,
        }
    }
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

enum EventBusOperation {
    Register {
        event_name: String,
        priority: EventPriority,
        update_pass: UpdatePass,
        with_update_pass_message: bool,
    },
    Unregister {
        event_name: String,
    },
    Subscribe {
        event_name: String,
        tunnel_id: u32,
        entity_id: Option<u32>,
    },
    Unsubscribe {
        tunnel_id: u32,
    },
}

pub struct EventBus {
    events: HashMap<String, Event>,
    operation_queue: VecDeque<EventBusOperation>,
    update_pass_map: HashMap<UpdatePass, BinaryHeap<MessageRequest>>,
    cached_requests: Vec<MessageRequestCache>,
    next_subscriber_id: AtomicU32,
    next_tunnel_id: AtomicU32,
    max_priority_locked: bool,
    current_update_pass: Option<UpdatePass>,
    current_message_request: Option<MessageRequest>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            operation_queue: VecDeque::from(vec![]),
            update_pass_map: HashMap::new(),
            cached_requests: vec![],
            next_subscriber_id: AtomicU32::new(0),
            next_tunnel_id: AtomicU32::new(0),
            max_priority_locked: false,
            current_update_pass: Some(UpdatePass::PreSim),
            current_message_request: None,
        }
    }

    pub fn lock_max_priority(&mut self) {
        self.process_operations();
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

    fn reinsert_stay_alive_requests(&mut self) {
        for message_request_cache in self.cached_requests.drain(..) {
            let update_pass = message_request_cache.update_pass;
            let message_request: MessageRequest = message_request_cache.into();

            // info!("Reinsert event: {}", message_request.event_name);

            self.update_pass_map
                .entry(update_pass)
                .or_insert_with(BinaryHeap::new)
                .push(message_request);
        }
    }

    fn process_operations(&mut self) {
        while let Some(operation) = self.operation_queue.pop_front() {
            match operation {
                EventBusOperation::Register {
                    event_name,
                    priority,
                    update_pass,
                    with_update_pass_message,
                } => {
                    if self.max_priority_locked && priority == EventPriority::Max {
                        panic!("Trying to register event at maximum priority which is locked.");
                    }

                    let event = Event {
                        name: event_name.clone(),
                        priority,
                        update_pass,
                        subscribers: vec![],
                        processed_subscribers: vec![],
                    };

                    match self.events.entry(event_name.clone()) {
                        Entry::Occupied(_) => {
                            warn!(
                                "You are trying to register an Event '{}' that already exists - Aborting!",
                                event_name
                            );
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(event);

                            let message_request = MessageRequestCache {
                                update_pass,
                                priority,
                                event_name: event_name.clone(),
                                stay_alive: with_update_pass_message,
                            };

                            info!("Registered event: {}", event_name);

                            self.cached_requests.push(message_request);
                        }
                    }
                }
                EventBusOperation::Unregister { event_name } => {
                    if let Some(event) = self.events.remove(&event_name) {
                        if let Some(message_heap) = self.update_pass_map.get_mut(&event.update_pass)
                        {
                            message_heap.retain(|e| e.event_name != event_name);
                            info!("Unregistered event: {}", event.name);
                        }
                    }
                }
                EventBusOperation::Subscribe {
                    event_name,
                    tunnel_id,
                    entity_id,
                } => {
                    if let Some(event) = self.events.get_mut(&event_name) {
                        self.add_subscriber(&event_name, tunnel_id, entity_id);
                        info!(
                            "Subscribed to event '{}' with tunnel_id {}",
                            event_name, tunnel_id
                        );
                    }
                }
                EventBusOperation::Unsubscribe { tunnel_id } => {
                    for event in self.events.values_mut() {
                        event
                            .subscribers
                            .retain(|subscriber| subscriber.tunnel_id != tunnel_id);
                    }

                    info!(
                        "Unsubscribed from event and closed tunnel with id: {}",
                        tunnel_id
                    );
                }
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl EventBus {
    pub fn is_ready(&self) -> bool {
        self.max_priority_locked
    }

    pub fn register(
        &mut self,
        event_name: String,
        priority: EventPriority,
        update_pass: UpdatePass,
        with_update_pass_message: bool,
    ) {
        self.operation_queue.push_back(EventBusOperation::Register {
            event_name,
            priority,
            update_pass,
            with_update_pass_message,
        });
    }

    pub fn unregister(&mut self, event_name: String) {
        self.operation_queue
            .push_back(EventBusOperation::Unregister { event_name });
    }

    /// @overload fun(self: table, eventName: string, ctxTable: table|nil, callbackFunc: function): integer
    pub fn subscribe(&mut self, event_name: String, entity_id: Option<u32>) -> u32 {
        let tunnel_id = self.next_tunnel_id.fetch_add(1, Ordering::SeqCst);
        self.operation_queue
            .push_back(EventBusOperation::Subscribe {
                event_name,
                tunnel_id,
                entity_id,
            });
        tunnel_id
    }

    pub fn unsubscribe(&mut self, tunnel_id: u32) {
        self.operation_queue
            .push_back(EventBusOperation::Unsubscribe { tunnel_id });
    }

    pub fn get_next_event(&mut self) -> Option<&EventData> {
        static mut EVENT_DATA_STORAGE: Option<EventData> = None;

        while let Some(update_pass) = self.current_update_pass {
            if let Some(queue) = self.update_pass_map.get_mut(&update_pass) {
                if self.current_message_request.is_none() {
                    if let Some(message_request) = queue.peek().cloned() {
                        self.current_message_request = Some(message_request.clone());

                        if message_request.stay_alive {
                            let message_request_cache = MessageRequestCache {
                                update_pass,
                                priority: message_request.priority,
                                event_name: message_request.event_name.clone(),
                                stay_alive: message_request.stay_alive,
                            };
                            self.cached_requests.push(message_request_cache);
                        }
                    }
                }

                if let Some(ref message_request) = self.current_message_request {
                    if let Some(event) = self.events.get_mut(&message_request.event_name) {
                        event.subscribers.sort_by(|a, b| a.id.cmp(&b.id));

                        if let Some(subscriber) = event.get_next_subscriber() {
                            let event_data = EventData {
                                update_pass,
                                event_type: EventType::SomeType,
                                tunnel_id: subscriber.tunnel_id,
                            };

                            unsafe {
                                EVENT_DATA_STORAGE = Some(event_data);
                            }

                            return unsafe { EVENT_DATA_STORAGE.as_ref() };
                        } else {
                            event.reset_processed_subscribers();
                            queue.pop();
                            self.current_message_request = None;
                        }
                    }
                }
            }

            let next_update_pass = {
                let mut iter = UpdatePass::iter().skip_while(|&pass| pass != update_pass);
                iter.next();
                iter.next()
            };

            self.current_update_pass = next_update_pass;
        }

        self.process_operations();
        self.reinsert_stay_alive_requests();
        self.current_update_pass = Some(UpdatePass::PreSim);
        None
    }

    pub fn print_update_pass_map(&self) {
        info!("Current state of update_pass_map:");

        // Create a sorted vector of UpdatePass keys based on the enum order
        let sorted_keys: Vec<_> = UpdatePass::iter().collect();

        for update_pass in sorted_keys {
            if let Some(message_heap) = self.update_pass_map.get(&update_pass) {
                info!("{:?}", update_pass);
                let mut messages: Vec<_> = message_heap.iter().cloned().collect();
                messages.sort();

                for message_request in messages {
                    if let Some(_event) = self.events.get(&message_request.event_name) {
                        info!(" - {:?}", message_request);
                    }
                }
            }
        }
    }
}
