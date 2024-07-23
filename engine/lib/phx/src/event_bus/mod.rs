use std::collections::{hash_map::Entry, BinaryHeap, HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};

use strum::IntoEnumIterator;
use internal::ConvertIntoString;
use tracing::{info, warn};

use event_data::*;
use event::*;
use frame_stage::*;
use frame_timer::*;
use message_request::*;

mod event_data;
mod event;
mod frame_stage;
mod frame_timer;
mod message_request;

enum EventBusOperation {
    Register {
        event_name: String,
        priority: i32,
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
    SetTimeScale {
        scale_factor: f64,
    },
}

pub struct EventBus {
    delta_time: f64,
    frame_timer: FrameTimer,
    frame_time_scale: f64,
    events: HashMap<String, Event>,
    operation_queue: VecDeque<EventBusOperation>,
    frame_stage_map: HashMap<FrameStage, BinaryHeap<MessageRequest>>,
    cached_requests: Vec<MessageRequestCache>,
    next_subscriber_id: AtomicU32,
    next_tunnel_id: AtomicU32,
    last_frame_stage: Option<FrameStage>,
    current_frame_stage: Option<FrameStage>,
    current_event: Option<Event>,
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
                priority: i32::MAX,
                frame_stage,
                subscribers: vec![],
                processed_subscribers: vec![],
            };

            let message_request = MessageRequestCache {
                frame_stage,
                priority: i32::MAX,
                event_name: event_name.clone(),
                stay_alive: true,
                for_entity_id: None,
            };

            events.insert(event_name, frame_stage_event);
            cached_requests.push(message_request);
        }

        Self {
            delta_time: 0.0,
            frame_timer: FrameTimer::new(),
            frame_time_scale: 1.0,
            events,
            operation_queue: VecDeque::new(),
            frame_stage_map: HashMap::new(),
            cached_requests,
            next_subscriber_id: AtomicU32::new(0),
            next_tunnel_id: AtomicU32::new(0),
            last_frame_stage: None,
            current_frame_stage: Some(FrameStage::PreSim),
            current_event: None,
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
            event.subscribers.sort_by(|a, b| a.id.cmp(&b.id));
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
                .or_default()
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
                EventBusOperation::SetTimeScale { scale_factor } => {
                    self.frame_time_scale = scale_factor;
                }
            }
        }
    }
}

#[luajit_ffi_gen::luajit_ffi(name = "EventBusImpl")]
impl EventBus {
    pub fn get_time_scale(&self) -> f64 {
        self.frame_time_scale
    }

    pub fn set_time_scale(&mut self, scale_factor: f64) {
        self.operation_queue
            .push_front(EventBusOperation::SetTimeScale { scale_factor });
    }

    pub fn register(
        &mut self,
        event_name: &str,
        priority: i32,
        frame_stage: FrameStage,
        with_frame_stage_message: bool,
    ) {
        if priority == i32::MAX {
            panic!("Trying to register event at highest priority which is reserved for frame stage events.");
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

        //info!("Entering get_next_event");

        if self.current_frame_stage.is_none() {
            self.current_frame_stage = Some(FrameStage::PreSim);
            //info!("Initializing current_frame_stage to PreSim");
        }

        if self.current_frame_stage != self.last_frame_stage {
            if let Some(frame_stage) = self.current_frame_stage {
                self.delta_time = self.frame_timer.update(frame_stage) * self.frame_time_scale;
                self.last_frame_stage = self.current_frame_stage;
            }
        }

        while let Some(frame_stage) = self.current_frame_stage {
            //info!("Processing frame stage: {:?}", frame_stage);

            if let Some(queue) = self.frame_stage_map.get_mut(&frame_stage) {
                //info!("Queue found for frame stage, length: {}", queue.len());

                if self.current_message_request.is_none() {
                    self.current_message_request = queue.pop();

                    if let Some(message_request) = &self.current_message_request {
                        //info!(
                        //    "Popped new message request: {:?}",
                        //    message_request.event_name
                        //);
                        if message_request.stay_alive {
                            //info!("Caching stay_alive message request");
                            let message_request_cache = MessageRequestCache {
                                frame_stage,
                                priority: message_request.priority,
                                event_name: message_request.event_name.clone(),
                                stay_alive: message_request.stay_alive,
                                for_entity_id: message_request.for_entity_id,
                            };
                            self.cached_requests.push(message_request_cache);
                        }
                    } else {
                        //info!("No more message requests in queue");
                    }
                }

                if let Some(ref message_request) = self.current_message_request {
                    if self.current_event.is_none() {
                        self.current_event =
                            self.events.get_mut(&message_request.event_name).cloned();
                        //info!(
                        //    "Retrieved event for message request: {:?}",
                        //    message_request.event_name
                        //);
                    }
                    if let Some(event) = &mut self.current_event {
                        if let Some(subscriber) = event.get_next_subscriber() {
                            //info!("Found next subscriber for event");
                            if message_request.stay_alive
                                || message_request.for_entity_id == subscriber.entity_id
                            {
                                let event_data = EventData {
                                    delta_time: self.delta_time,
                                    frame_stage,
                                    tunnel_id: subscriber.tunnel_id,
                                };

                                unsafe {
                                    EVENT_DATA_STORAGE = Some(event_data);
                                }

                                //info!(
                                //    "Returning event data for frame stage {:?}, tunnel_id {:?}",
                                //    frame_stage, subscriber.tunnel_id
                                //);
                                return unsafe { EVENT_DATA_STORAGE.as_ref() };
                            }
                        } else {
                            //info!("No more subscribers for current event");
                            event.reset_processed_subscribers();
                            self.current_event = None;
                            self.current_message_request = None;
                        }
                    }
                }

                if self.current_message_request.is_none() && queue.is_empty() {
                    //info!(
                    //    "No more message requests and queue is empty, moving to next frame stage"
                    //);
                    let next_frame_stage = {
                        let mut iter = FrameStage::iter().skip_while(|&pass| pass != frame_stage);
                        iter.next();
                        iter.next()
                    };
                    self.current_frame_stage = next_frame_stage;
                    //info!("Next frame stage set to {:?}", self.current_frame_stage);
                }
            } else {
                //info!("No queue for current frame stage, moving to next");
                let next_frame_stage = {
                    let mut iter = FrameStage::iter().skip_while(|&pass| pass != frame_stage);
                    iter.next();
                    iter.next()
                };
                self.current_frame_stage = next_frame_stage;
                //info!("Next frame stage set to {:?}", self.current_frame_stage);
            }
        }

        //info!("All frame stages processed");
        self.process_operations();
        //info!("Operations processed");
        self.reinsert_stay_alive_requests();
        //info!("Stay-alive requests reinserted");
        self.current_frame_stage = Some(FrameStage::PreSim);
        //info!("Frame stage reset to PreSim");
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
