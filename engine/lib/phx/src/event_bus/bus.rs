use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};

use strum::IntoEnumIterator;
use tracing::warn;

use super::{Event, EventData, EventPayload, FrameStage, FrameTimer, Subscriber};

pub type EventId = u16;
pub type EntityId = u64;
pub type TunnelId = u32;

#[derive(Debug, Clone)]
enum EventBusOperation {
    Register {
        event_id: EventId,
        event_name: String,
        frame_stage: FrameStage,
        with_frame_stage_message: bool,
    },
    Unregister {
        event_id: EventId,
    },
    Subscribe {
        event_id: EventId,
        tunnel_id: TunnelId,
        entity_id: Option<EntityId>,
    },
    Unsubscribe {
        tunnel_id: TunnelId,
    },
    Send {
        event_id: EventId,
        entity_id: Option<EntityId>,
        payload: Option<EventPayload>,
    },
    SetTimeScale {
        scale_factor: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct MessageRequest {
    event_id: EventId,
    stay_alive: bool,
    for_entity_id: Option<EntityId>,
    payload: Option<EventPayload>,
}

pub struct EventBus {
    delta_time: f64,
    frame_timer: FrameTimer,
    frame_time_scale: f64,
    events: HashMap<EventId, Event>,
    operation_queue: VecDeque<EventBusOperation>,
    frame_stage_requests: HashMap<FrameStage, Vec<MessageRequest>>,
    cached_requests: Vec<(FrameStage, MessageRequest)>,
    next_tunnel_id: AtomicU32,
    prev_frame_stage: FrameStage,
    current_frame_stage: FrameStage,
    current_message_request: Option<MessageRequest>,
}

impl EventBus {
    pub fn new() -> Self {
        let events = HashMap::new();
        let cached_requests = Vec::new();

        // Create an event for every frame stage
        // for frame_stage in FrameStage::iter() {
        //     let event_name = format!("{:?}", frame_stage);
        //     let event = Event::new(event_name.clone(), i32::MAX, frame_stage);

        //     let message_request = MessageRequest {
        //         event_id: event_name.clone(),
        //         stay_alive: true,
        //         for_entity_id: None,
        //         payload: None,
        //     };

        //     events.insert(event_name, event);
        //     cached_requests.push((frame_stage, message_request));
        // }

        Self {
            delta_time: 0.0,
            frame_timer: FrameTimer::new(),
            frame_time_scale: 1.0,
            events,
            operation_queue: VecDeque::new(),
            frame_stage_requests: HashMap::new(),
            cached_requests,
            next_tunnel_id: AtomicU32::new(0),
            prev_frame_stage: FrameStage::last(), // to trigger delta time recalculation of the first stage for the new frame
            current_frame_stage: FrameStage::first(),
            current_message_request: None,
        }
    }

    fn add_subscriber(
        &mut self,
        event_id: EventId,
        tunnel_id: TunnelId,
        entity_id: Option<EntityId>,
    ) {
        let Some(event) = self.events.get_mut(&event_id) else {
            panic!("error while pushing subscriber");
        };

        let subscriber = Subscriber::new(tunnel_id, entity_id);
        event.add_subscriber(subscriber);
    }

    fn reinsert_stay_alive_requests(&mut self) {
        println!("Reinsert stay-alive requests");
        // NOTE: we reinsert requests in reverse order so later processing will pop them in correct one
        while let Some((frame_stage, message_request)) = self.cached_requests.pop() {
            println!("  {frame_stage:?}: {message_request:?}");

            self.frame_stage_requests
                .entry(frame_stage)
                .or_default()
                .push(message_request);
        }
        println!("Stay-alive requests reinserted");
    }

    fn process_operations(&mut self) {
        println!("Process operations");
        while let Some(operation) = self.operation_queue.pop_front() {
            println!("  {operation:?}");
            match operation {
                EventBusOperation::Register {
                    event_id,
                    event_name,
                    frame_stage,
                    with_frame_stage_message,
                } => {
                    match self.events.entry(event_id) {
                        Entry::Occupied(_) => {
                            // TODO: panic?
                            warn!("You are trying to register an Event '{event_name}':{event_id} that already exists - Aborting!");
                        }
                        Entry::Vacant(entry) => {
                            let event = Event::new(event_id, &event_name, frame_stage);

                            entry.insert(event);

                            if with_frame_stage_message {
                                let message_request = MessageRequest {
                                    event_id,
                                    stay_alive: with_frame_stage_message,
                                    for_entity_id: None,
                                    payload: None,
                                };

                                self.cached_requests.push((frame_stage, message_request));
                            }
                        }
                    }
                }
                EventBusOperation::Unregister { event_id } => {
                    if let Some(event) = self.events.remove(&event_id) {
                        if let Some(message_requests) =
                            self.frame_stage_requests.get_mut(&event.frame_stage())
                        {
                            message_requests.retain(|e| e.event_id != event_id);
                        }
                    } else {
                        // TODO: unsubscribing from unknown event. Warning?
                    }
                }
                EventBusOperation::Subscribe {
                    event_id,
                    tunnel_id,
                    entity_id,
                } => {
                    if let Some(event) = self.events.get(&event_id) {
                        let event_name = event.name().to_string();
                        self.add_subscriber(event_id, tunnel_id, entity_id);
                        println!("    Subscribed to event: {event_name}");
                    }
                }
                EventBusOperation::Unsubscribe { tunnel_id } => {
                    self.events
                        .values_mut()
                        .for_each(|event| event.remove_subscriber(tunnel_id));
                }
                EventBusOperation::Send {
                    event_id,
                    entity_id,
                    payload,
                } => {
                    if let Some(event) = self.events.get(&event_id) {
                        let message_request = MessageRequest {
                            event_id: event.id(),
                            stay_alive: false,
                            for_entity_id: entity_id,
                            payload,
                        };

                        self.cached_requests
                            .push((event.frame_stage(), message_request));
                        println!("    Event: {}", event.name());
                    }
                }
                EventBusOperation::SetTimeScale { scale_factor } => {
                    self.frame_time_scale = scale_factor;
                }
            }
        }
        println!("Operations processed");
    }
}

#[luajit_ffi_gen::luajit_ffi]
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
        event_id: u16,
        event_name: &str,
        frame_stage: FrameStage,
        with_frame_stage_message: bool,
    ) {
        self.operation_queue.push_back(EventBusOperation::Register {
            event_id,
            event_name: event_name.into(),
            frame_stage,
            with_frame_stage_message,
        });
    }

    pub fn unregister(&mut self, event_id: u16) {
        self.operation_queue
            .push_back(EventBusOperation::Unregister { event_id });
    }

    /// @overload fun(self: table, eventName: string, ctxTable: table|nil, callbackFunc: function): integer
    pub fn subscribe(&mut self, event_id: u16, entity_id: Option<u64>) -> u32 {
        let tunnel_id = self.next_tunnel_id.fetch_add(1, Ordering::SeqCst);
        self.operation_queue
            .push_back(EventBusOperation::Subscribe {
                event_id,
                tunnel_id,
                entity_id,
            });
        tunnel_id
    }

    pub fn unsubscribe(&mut self, tunnel_id: u32) {
        self.operation_queue
            .push_back(EventBusOperation::Unsubscribe { tunnel_id });
    }

    /// @overload fun(self: table, eventName: string, ctxTable: table|nil, payload: EventPayload|nil)
    pub fn send(&mut self, event_id: u16, entity_id: Option<u64>, payload: Option<&EventPayload>) {
        self.operation_queue.push_back(EventBusOperation::Send {
            event_id,
            entity_id,
            payload: payload.cloned(),
        })
    }

    pub fn start_event_iteration(&mut self) {
        self.process_operations();
        self.reinsert_stay_alive_requests();
        self.current_frame_stage = FrameStage::first();
        println!("Frame stage reset to PreSim");
    }

    /// Iterates over events of the frame.
    /// Returns `None`/`nil` when there are no more events.
    pub fn next_event(&mut self) -> Option<EventData> {
        println!("Entering next_event");

        if self.current_frame_stage != self.prev_frame_stage {
            self.delta_time =
                self.frame_timer.update(self.current_frame_stage) * self.frame_time_scale;
            self.prev_frame_stage = self.current_frame_stage;
        }

        loop {
            println!("  Processing frame stage: {:?}", self.current_frame_stage);

            if let Some(message_requests) =
                self.frame_stage_requests.get_mut(&self.current_frame_stage)
            {
                println!(
                    "    Queue found for frame stage, length: {}",
                    message_requests.len()
                );

                if self.current_message_request.is_none() {
                    // NOTE: pop will return messages in correct order because they were inserted in reverse one in reinsert_stay_alive_requests method
                    self.current_message_request = message_requests.pop();

                    if let Some(message_request) = &self.current_message_request {
                        println!(
                            "      Popped new message request. Event id {:?}, entity id: {:?}, stay alive: {}",
                            message_request.event_id, message_request.for_entity_id, message_request.stay_alive
                        );
                        if message_request.stay_alive {
                            println!("        Caching stay_alive message request");
                            self.cached_requests
                                .push((self.current_frame_stage, message_request.clone()));
                        }
                    } else {
                        println!("      No more message requests in queue");
                    }
                }

                if let Some(message_request) = &self.current_message_request {
                    println!(
                        "      Retrieved event for message request. Event id: {:?}, entity id: {:?}, stay alive: {}",
                        message_request.event_id, message_request.for_entity_id, message_request.stay_alive
                    );

                    let current_event = self.events.get_mut(&message_request.event_id);
                    if let Some(event) = current_event {
                        let frame_stage = event.frame_stage();
                        if let Some(subscriber) = event.next_subscriber() {
                            println!("        Found next subscriber for event. Tunnel id: {}, entity id: {:?}", subscriber.tunnel_id(), subscriber.entity_id());
                            if message_request.stay_alive
                                || message_request.for_entity_id == subscriber.entity_id()
                            {
                                let event_data = EventData::new(
                                    self.delta_time,
                                    self.current_frame_stage,
                                    subscriber.tunnel_id(),
                                    message_request.payload.clone(),
                                );

                                println!(
                                   "          => Returning event data for frame stage {frame_stage:?}, tunnel_id {:?}",
                                   subscriber.tunnel_id()
                                );

                                return Some(event_data);
                            }
                        } else {
                            println!("        No more subscribers for current event");
                            self.current_message_request = None;
                        }
                    }
                }

                if self.current_message_request.is_none() && message_requests.is_empty() {
                    println!(
                        "    No more message requests and queue is empty, moving to next frame stage"
                    );
                    if let Some(next_frame_stage) = self.current_frame_stage.next() {
                        self.current_frame_stage = next_frame_stage;
                        println!("      Next frame stage set to {next_frame_stage:?}");
                    } else {
                        println!(
                            "      The last stage - finish events processing by returning None"
                        );
                        break;
                    }
                }
            } else {
                println!("    No queue for current frame stage, moving to next");
                if let Some(next_frame_stage) = self.current_frame_stage.next() {
                    self.current_frame_stage = next_frame_stage;
                    println!("      Next frame stage set to {next_frame_stage:?}");
                } else {
                    println!("      The last stage - finish events processing by returning None");
                    break;
                }
            }
        }

        println!("All frame stages processed");
        None
    }

    pub fn print_frame_stage_map(&self) {
        println!("Current state of frame_stage_map:");

        // Create a sorted vector of FrameStage keys based on the enum order
        for frame_stage in FrameStage::iter() {
            if let Some(message_requests) = self.frame_stage_requests.get(&frame_stage) {
                println!("  {frame_stage:?}");

                for message_request in message_requests {
                    if let Some(_event) = self.events.get(&message_request.event_id) {
                        println!("   - {message_request:?}");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EntityId, EventBus, EventId, TunnelId};
    use crate::event_bus::{EventPayload, FrameStage};

    fn test_event_bus(
        events: &[(EventId, FrameStage)],
        subscribes: &[(EventId, Option<EntityId>)],
        sends: &[(EventId, Option<EntityId>, Option<EventPayload>)],
        expected: &[(FrameStage, TunnelId, Option<EventPayload>)],
    ) {
        let mut event_bus = EventBus::new();

        events.iter().for_each(|e| {
            let event_name = format!("TestEvent{}", e.0);
            event_bus.register(e.0, &event_name, e.1, false);
        });

        let tunnel_ids: Vec<_> = subscribes
            .iter()
            .map(|s| event_bus.subscribe(s.0, s.1))
            .collect();

        sends
            .iter()
            .for_each(|s| event_bus.send(s.0, s.1, s.2.as_ref()));

        event_bus.start_event_iteration();

        expected.iter().enumerate().for_each(|(i, e)| {
            let event = event_bus
                .next_event()
                .unwrap_or_else(|| panic!("Event {i} was not sent"));

            assert!(tunnel_ids.contains(&e.1), "Unexpected tunnel id: {}", e.1);
            assert_eq!(event.frame_stage(), e.0, "Frame stage");
            assert_eq!(event.tunnel_id(), e.1, "Tunnel id");
            assert_eq!(event.payload(), e.2.as_ref(), "Payload");
        });

        let next_event = event_bus.next_event();
        assert!(
            next_event.is_none(),
            "There are more events than expected. Next event: {next_event:?}"
        );
    }

    #[test]
    fn test_event_bus_one_event_one_subscriber() {
        test_event_bus(
            // one event
            &[(0, FrameStage::first())],
            // one subscriber
            &[(0, Some(0))],
            // send event once
            &[(0, Some(0), None)],
            // subscriber receives an event message
            &[(FrameStage::first(), 0, None)],
        );
    }

    #[test]
    fn test_event_bus_one_event_two_subscribers() {
        test_event_bus(
            // one event
            &[(0, FrameStage::first())],
            // two subscribers
            &[(0, Some(0)), (0, Some(0))],
            // send event once
            &[(0, Some(0), None)],
            // each subscriber receive an event message
            &[
                (FrameStage::first(), 0, None),
                (FrameStage::first(), 1, None),
            ],
        );
    }

    #[test]
    fn test_event_bus_two_events_two_subscribers_two_stages() {
        test_event_bus(
            // two events
            &[(0, FrameStage::first()), (1, FrameStage::last())],
            // two subscribers
            &[(0, Some(0)), (1, Some(1))],
            // send each event once
            &[(0, Some(0), None), (1, Some(1), None)],
            // each subscriber receive its own event message
            &[
                (FrameStage::first(), 0, None),
                (FrameStage::last(), 1, None),
            ],
        );
    }

    #[test]
    fn test_event_bus_one_event_two_subscribers_payload() {
        test_event_bus(
            // one event
            &[(0, FrameStage::first())],
            // two subscribers
            &[(0, Some(0)), (0, Some(0))],
            // send event with payload
            &[(0, Some(0), Some(EventPayload::U16(42)))],
            // each subscriber receive an event message with the same payload
            &[
                (FrameStage::first(), 0, Some(EventPayload::U16(42))),
                (FrameStage::first(), 1, Some(EventPayload::U16(42))),
            ],
        );
    }

    #[test]
    fn test_event_bus_one_event_two_subscribers_two_entities() {
        test_event_bus(
            // one event
            &[(0, FrameStage::first())],
            // two subscribers: one for a specific entity, another for global events
            &[(0, Some(0)), (0, None)],
            // send event twice: first with entity id, second without (global event)
            &[(0, Some(0), None), (0, None, None)],
            // first subscriber receives one event message with specific entity, second receives two event messages for all entities
            &[
                (FrameStage::first(), 0, None),
                (FrameStage::first(), 1, None),
            ],
        );
    }

    #[test]
    fn test_event_bus_one_event_two_subscribers_global() {
        test_event_bus(
            // one event
            &[(0, FrameStage::first())],
            // two subscribers: one for a specific entity, another for global events
            &[(0, Some(0)), (0, None)],
            // send event twice: first with entity id, second without (global event)
            &[(0, None, Some(EventPayload::Bool(true))), (0, None, None)],
            // first subscriber doesn't receive any event messages, second receive two
            &[
                (FrameStage::first(), 1, Some(EventPayload::Bool(true))),
                (FrameStage::first(), 1, None),
            ],
        );
    }
}
