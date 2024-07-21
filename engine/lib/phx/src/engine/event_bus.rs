use std::collections::{hash_map::Entry, BinaryHeap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use internal::ConvertIntoString;
use tracing::{info, warn};

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter)]
pub enum FrameStage {
    // Before physics update
    PreSim,
    // Physics update
    Sim,
    // After physics update
    PostSim,
    // Before frame render
    PreRender,
    // Frame render
    Render,
    // After frame render
    PostRender,
    // Before input handling
    PreInput,
    // Input handling
    Input,
    // After input handling
    PostInput,
}

impl Default for FrameStage {
    fn default() -> Self {
        FrameStage::PreRender
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Subscriber {
    id: u32,
    tunnel_id: u32,
    entity_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub priority: EventPriority,
    pub frame_stage: FrameStage,
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
    pub frame_stage: FrameStage,
    pub tunnel_id: u32,
}

#[luajit_ffi_gen::luajit_ffi]
impl EventData {
    pub fn get_frame_stage(&self) -> FrameStage {
        self.frame_stage
    }

    pub fn get_tunnel_id(&self) -> u32 {
        self.tunnel_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MessageRequestCache {
    frame_stage: FrameStage,
    priority: EventPriority,
    event_name: String,
    stay_alive: bool,
    for_entity_id: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MessageRequest {
    priority: EventPriority,
    event_name: String,
    stay_alive: bool,
    for_entity_id: Option<u64>,
}

impl From<MessageRequestCache> for MessageRequest {
    fn from(cache: MessageRequestCache) -> Self {
        MessageRequest {
            priority: cache.priority,
            event_name: cache.event_name,
            stay_alive: cache.stay_alive,
            for_entity_id: cache.for_entity_id,
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
        frame_stage: FrameStage,
        with_frame_stage_message: bool,
    },
    Unregister {
        event_name: String,
    },
    Subscribe {
        event_name: String,
        tunnel_id: u32,
        entity_id: Option<u64>,
    },
    Unsubscribe {
        tunnel_id: u32,
    },
    Send {
        event_name: String,
        entity_id: u64,
    },
}

pub struct EventBus {
    events: HashMap<String, Event>,
    operation_queue: VecDeque<EventBusOperation>,
    frame_stage_map: HashMap<FrameStage, BinaryHeap<MessageRequest>>,
    cached_requests: Vec<MessageRequestCache>,
    next_subscriber_id: AtomicU32,
    next_tunnel_id: AtomicU32,
    current_frame_stage: Option<FrameStage>,
    current_message_request: Option<MessageRequest>,
}

impl EventBus {
    pub fn new() -> Self {
        let mut events: HashMap<String, Event> = HashMap::new();
        let mut cached_requests: Vec<MessageRequestCache> = Vec::new();

        // Create an event for every frame stage and set it at max priority
        for frame_stage in FrameStage::iter() {
            let event_name = format!("{:?}", frame_stage);
            let frame_stage_event = Event {
                name: event_name.clone(),
                priority: EventPriority::Max,
                frame_stage,
                subscribers: vec![],
                processed_subscribers: vec![],
            };

            let message_request = MessageRequestCache {
                frame_stage,
                priority: EventPriority::Max,
                event_name: event_name.clone(),
                stay_alive: true,
                for_entity_id: None,
            };

            events.insert(event_name, frame_stage_event);
            cached_requests.push(message_request);
        }

        Self {
            events,
            operation_queue: VecDeque::new(),
            frame_stage_map: HashMap::new(),
            cached_requests,
            next_subscriber_id: AtomicU32::new(0),
            next_tunnel_id: AtomicU32::new(0),
            current_frame_stage: Some(FrameStage::PreSim),
            current_message_request: None,
        }
    }

    pub fn add_subscriber(&mut self, event_name: &str, tunnel_id: u32, entity_id: Option<u64>) {
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
            let frame_stage = message_request_cache.frame_stage;
            let message_request: MessageRequest = message_request_cache.into();

            // info!("Reinsert event: {}", message_request.event_name);

            self.frame_stage_map
                .entry(frame_stage)
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
                    frame_stage,
                    with_frame_stage_message,
                } => {
                    let event = Event {
                        name: event_name.clone(),
                        priority,
                        frame_stage,
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

                            if with_frame_stage_message {
                                let message_request = MessageRequestCache {
                                    frame_stage,
                                    priority,
                                    event_name: event_name.clone(),
                                    stay_alive: with_frame_stage_message,
                                    for_entity_id: None,
                                };

                                self.cached_requests.push(message_request);
                            }
                            info!("Registered event: {}", event_name);
                        }
                    }
                }
                EventBusOperation::Unregister { event_name } => {
                    if let Some(event) = self.events.remove(&event_name) {
                        if let Some(message_heap) = self.frame_stage_map.get_mut(&event.frame_stage)
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
                    if let Some(_event) = self.events.get_mut(&event_name) {
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
                EventBusOperation::Send {
                    event_name,
                    entity_id,
                } => {
                    if let Some(event) = self.events.get(&event_name) {
                        let message_request = MessageRequestCache {
                            frame_stage: event.frame_stage,
                            priority: event.priority,
                            event_name: event.name.clone(),
                            stay_alive: false,
                            for_entity_id: Some(entity_id),
                        };

                        self.cached_requests.push(message_request);
                    }
                }
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl EventBus {
    pub fn register(
        &mut self,
        event_name: &str,
        priority: EventPriority,
        frame_stage: FrameStage,
        with_frame_stage_message: bool,
    ) {
        if priority == EventPriority::Max {
            panic!("Trying to register event at maximum priority which is locked.");
        }

        self.operation_queue.push_back(EventBusOperation::Register {
            event_name: event_name.to_string(),
            priority,
            frame_stage,
            with_frame_stage_message,
        });
    }

    pub fn unregister(&mut self, event_name: &str) {
        self.operation_queue
            .push_back(EventBusOperation::Unregister {
                event_name: event_name.to_string(),
            });
    }

    /// @overload fun(self: table, eventName: string, ctxTable: table|nil, callbackFunc: function): integer
    pub fn subscribe(&mut self, event_name: &str, entity_id: Option<u64>) -> u32 {
        let tunnel_id = self.next_tunnel_id.fetch_add(1, Ordering::SeqCst);
        self.operation_queue
            .push_back(EventBusOperation::Subscribe {
                event_name: event_name.to_string(),
                tunnel_id,
                entity_id,
            });
        tunnel_id
    }

    pub fn unsubscribe(&mut self, tunnel_id: u32) {
        self.operation_queue
            .push_back(EventBusOperation::Unsubscribe { tunnel_id });
    }

    /// @overload fun(self: table, eventName: string, ctxTable: table|nil)
    pub fn send(&mut self, event_name: &str, entity_id: u64) {
        self.operation_queue.push_back(EventBusOperation::Send {
            event_name: event_name.to_string(),
            entity_id,
        })
    }

    pub fn get_next_event(&mut self) -> Option<&EventData> {
        static mut EVENT_DATA_STORAGE: Option<EventData> = None;

        while let Some(frame_stage) = self.current_frame_stage {
            if let Some(queue) = self.frame_stage_map.get_mut(&frame_stage) {
                if self.current_message_request.is_none() {
                    if let Some(message_request) = queue.peek().cloned() {
                        self.current_message_request = Some(message_request.clone());

                        if message_request.stay_alive {
                            let message_request_cache = MessageRequestCache {
                                frame_stage,
                                priority: message_request.priority,
                                event_name: message_request.event_name.clone(),
                                stay_alive: message_request.stay_alive,
                                for_entity_id: message_request.for_entity_id,
                            };
                            self.cached_requests.push(message_request_cache);
                        }
                    }
                }

                if let Some(ref message_request) = self.current_message_request {
                    if let Some(event) = self.events.get_mut(&message_request.event_name) {
                        event.subscribers.sort_by(|a, b| a.id.cmp(&b.id));

                        if let Some(subscriber) = event.get_next_subscriber() {
                            if message_request.stay_alive
                                || message_request.for_entity_id == subscriber.entity_id
                            {
                                let event_data = EventData {
                                    frame_stage,
                                    tunnel_id: subscriber.tunnel_id,
                                };

                                unsafe {
                                    EVENT_DATA_STORAGE = Some(event_data);
                                }

                                return unsafe { EVENT_DATA_STORAGE.as_ref() };
                            }
                        } else {
                            event.reset_processed_subscribers();
                            queue.pop();
                            self.current_message_request = None;
                        }
                    }
                }
            }

            let next_frame_stage = {
                let mut iter = FrameStage::iter().skip_while(|&pass| pass != frame_stage);
                iter.next();
                iter.next()
            };

            self.current_frame_stage = next_frame_stage;
        }

        self.process_operations();
        self.reinsert_stay_alive_requests();
        self.current_frame_stage = Some(FrameStage::PreSim);
        None
    }

    pub fn print_frame_stage_map(&self) {
        info!("Current state of frame_stage_map:");

        // Create a sorted vector of FrameStage keys based on the enum order
        let sorted_keys: Vec<_> = FrameStage::iter().collect();

        for frame_stage in sorted_keys {
            if let Some(message_heap) = self.frame_stage_map.get(&frame_stage) {
                info!("{:?}", frame_stage);
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