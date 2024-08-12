use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use strum::IntoEnumIterator;
use tracing::warn;

use super::{EventData, EventMessage, EventPayload, FrameStage, FrameTimer, Subscriber};

pub type EventId = u16;
pub type EntityId = u64;
pub type TunnelId = u32;

#[derive(Debug, Clone)]
enum EventBusOperation {
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
    keep_alive: bool,
    for_entity_id: Option<EntityId>,
    payload: Option<EventPayload>,
}

pub struct EventBus {
    delta_time: f64,
    frame_timer: FrameTimer,
    frame_time_scale: f64,
    event_messages: HashMap<EventId, EventMessage>,
    operations: Vec<EventBusOperation>,
    frame_stage_requests: Vec<Vec<MessageRequest>>,
    cached_requests: Vec<Vec<MessageRequest>>,
    next_tunnel_id: AtomicU32,
    prev_frame_stage: FrameStage,
    current_frame_stage: FrameStage,
    current_message_request: Option<MessageRequest>,
}

impl EventBus {
    pub fn new() -> Self {
        let mut event_messages = HashMap::new();
        let mut cached_requests: Vec<Vec<MessageRequest>> =
            vec![Default::default(); FrameStage::len()];

        // Create an event for every frame stage
        for frame_stage in FrameStage::iter() {
            let event_type = &frame_stage.as_event_type();
            let event_id = event_type.index();
            let event_message =
                EventMessage::new(event_id, &event_type.to_string(), frame_stage, false);

            let message_request = MessageRequest {
                event_id,
                keep_alive: true,
                for_entity_id: None,
                payload: None,
            };

            event_messages.insert(event_id, event_message);
            cached_requests[frame_stage.index()].push(message_request);
        }

        Self {
            delta_time: 0.0,
            frame_timer: FrameTimer::new(),
            frame_time_scale: 1.0,
            event_messages,
            operations: vec![],
            frame_stage_requests: vec![Default::default(); FrameStage::len()],
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
        let event = self
            .event_messages
            .get_mut(&event_id)
            .expect("error while adding subscriber");
        let subscriber = Subscriber::new(tunnel_id, entity_id);
        event.add_subscriber(subscriber);
    }

    fn process_operations(&mut self) {
        // println!("Process operations");

        let mut operations = vec![];
        std::mem::swap(&mut operations, &mut self.operations);

        for operation in operations.drain(..) {
            // println!("  {operation:?}");
            match operation {
                EventBusOperation::Unregister { event_id } => {
                    if let Some(event) = self.event_messages.remove(&event_id) {
                        if let Some(message_requests) = self
                            .frame_stage_requests
                            .get_mut(event.frame_stage().index())
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
                    if self.event_messages.contains_key(&event_id) {
                        self.add_subscriber(event_id, tunnel_id, entity_id);
                    }
                }
                EventBusOperation::Unsubscribe { tunnel_id } => {
                    self.event_messages
                        .values_mut()
                        .for_each(|event| event.remove_subscriber(tunnel_id));
                }
                EventBusOperation::Send {
                    event_id,
                    entity_id,
                    payload,
                } => {
                    if let Some(event) = self.event_messages.get(&event_id) {
                        let message_request = MessageRequest {
                            event_id: event.id(),
                            keep_alive: false,
                            for_entity_id: entity_id,
                            payload,
                        };

                        // NOTE: we insert requests in reverse order so later processing will pop them in the correct one
                        self.cached_requests[event.frame_stage().index()]
                            .insert(0, message_request);
                        // println!("    Event: {}", event.name());
                    }
                }
                EventBusOperation::SetTimeScale { scale_factor } => {
                    self.frame_time_scale = scale_factor;
                }
            }
        }
        // println!("Operations processed");
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl EventBus {
    pub fn get_time_scale(&self) -> f64 {
        self.frame_time_scale
    }

    pub fn set_time_scale(&mut self, scale_factor: f64) {
        // TODO: use additional variable instead of operation?
        self.operations
            .insert(0, EventBusOperation::SetTimeScale { scale_factor });
    }

    pub fn has_rust_payload(&self, event_id: u16) -> bool {
        let event_message = self
            .event_messages
            .get(&event_id)
            .unwrap_or_else(|| panic!("Unknown event: {event_id}"));
        event_message.has_rust_payload()
    }

    pub fn register(
        &mut self,
        event_id: u16,
        event_name: &str,
        frame_stage: FrameStage,
        rust_payload: bool,
    ) {
        // NOTE: we register event immediately instead of via operation to make `has_rust_payload` method work properly
        match self.event_messages.entry(event_id) {
            Entry::Occupied(_) => {
                // TODO: panic?
                warn!("You are trying to register an Event '{event_name}':{event_id} that already exists - Aborting!");
            }
            Entry::Vacant(entry) => {
                let event_message =
                    EventMessage::new(event_id, event_name, frame_stage, rust_payload);

                entry.insert(event_message);
            }
        }
    }

    pub fn unregister(&mut self, event_id: u16) {
        self.operations
            .push(EventBusOperation::Unregister { event_id });
    }

    /// @overload fun(self: table, eventType: integer, ctxTable: table|nil, callbackFunc: function): integer
    pub fn subscribe(&mut self, event_id: u16, entity_id: Option<u64>) -> u32 {
        let tunnel_id = self.next_tunnel_id.fetch_add(1, Ordering::SeqCst);
        self.operations.push(EventBusOperation::Subscribe {
            event_id,
            tunnel_id,
            entity_id,
        });
        tunnel_id
    }

    pub fn unsubscribe(&mut self, tunnel_id: u32) {
        self.operations
            .push(EventBusOperation::Unsubscribe { tunnel_id });
    }

    /// @overload fun(self: table, eventType: integer, ctxTable: table|nil, payload: EventPayload|nil)
    pub fn send(&mut self, event_id: u16, entity_id: Option<u64>, payload: Option<&EventPayload>) {
        self.operations.push(EventBusOperation::Send {
            event_id,
            entity_id,
            payload: payload.cloned(),
        })
    }

    pub fn start_event_iteration(&mut self) {
        assert!(
            self.frame_stage_requests.iter().all(|m| m.is_empty()),
            "All events of the previous frame should be processed"
        );

        self.process_operations();

        std::mem::swap(&mut self.frame_stage_requests, &mut self.cached_requests);
        // println!("Cached requests were transferred");

        self.current_frame_stage = FrameStage::first();
        // println!("Frame stage reset to PreSim");
    }

    /// Iterates over events of the frame.
    /// Returns `None`/`nil` when there are no more events.
    pub fn next_event(&mut self) -> Option<EventData> {
        // println!("Entering next_event");

        if self.current_frame_stage != self.prev_frame_stage {
            self.delta_time =
                self.frame_timer.update(self.current_frame_stage) * self.frame_time_scale;
            self.prev_frame_stage = self.current_frame_stage;
        }

        loop {
            // println!("  Processing frame stage: {:?}", self.current_frame_stage);

            if let Some(message_requests) = self
                .frame_stage_requests
                .get_mut(self.current_frame_stage.index())
            {
                // println!(
                //     "    Queue found for frame stage, length: {}",
                //     message_requests.len()
                // );

                if self.current_message_request.is_none() {
                    // NOTE: pop will return messages in correct order because they were inserted in reverse one in reinsert_stay_alive_requests method
                    self.current_message_request = message_requests.pop();

                    if let Some(message_request) = &self.current_message_request {
                        // println!(
                        //     "      Popped new message request. Event id {:?}, entity id: {:?}",
                        //     message_request.event_id, message_request.for_entity_id
                        // );

                        if message_request.keep_alive {
                            // println!("        Caching keep_alive message request");
                            self.cached_requests[self.current_frame_stage.index()]
                                .push(message_request.clone());
                        }
                    } else {
                        // println!("      No more message requests in queue");
                    }
                }

                if let Some(message_request) = &self.current_message_request {
                    // println!(
                    //     "      Retrieved event for message request. Event id: {:?}, entity id: {:?}",
                    //     message_request.event_id, message_request.for_entity_id
                    // );

                    let current_event = self.event_messages.get_mut(&message_request.event_id);
                    if let Some(event) = current_event {
                        // let frame_stage = event.frame_stage();
                        if let Some(subscriber) = event.next_subscriber() {
                            // println!("        Found next subscriber for event. Tunnel id: {}, entity id: {:?}", subscriber.tunnel_id(), subscriber.entity_id());
                            if message_request.keep_alive
                                || message_request.for_entity_id == subscriber.entity_id()
                            {
                                let event_data = EventData::new(
                                    self.delta_time,
                                    self.current_frame_stage,
                                    subscriber.tunnel_id(),
                                    message_request.payload.clone(),
                                );

                                // println!(
                                //    "          => Returning event data for frame stage {frame_stage:?}, tunnel_id {:?}",
                                //    subscriber.tunnel_id()
                                // );

                                return Some(event_data);
                            }
                        } else {
                            // println!("        No more subscribers for current event");
                            self.current_message_request = None;
                        }
                    }
                }

                if self.current_message_request.is_none() && message_requests.is_empty() {
                    // println!(
                    //     "    No more message requests and queue is empty, moving to next frame stage"
                    // );
                    if let Some(next_frame_stage) = self.current_frame_stage.next() {
                        self.current_frame_stage = next_frame_stage;
                        // println!("      Next frame stage set to {next_frame_stage:?}");
                    } else {
                        // println!(
                        //     "      The last stage - finish events processing by returning None"
                        // );
                        break;
                    }
                }
            } else {
                // println!("    No queue for current frame stage, moving to next");
                if let Some(next_frame_stage) = self.current_frame_stage.next() {
                    self.current_frame_stage = next_frame_stage;
                    // println!("      Next frame stage set to {next_frame_stage:?}");
                } else {
                    // println!("      The last stage - finish events processing by returning None");
                    break;
                }
            }
        }

        // println!("All frame stages processed");
        None
    }

    pub fn print_frame_stage_map(&self) {
        println!("Current state of frame_stage_map:");

        // Create a sorted vector of FrameStage keys based on the enum order
        for frame_stage in FrameStage::iter() {
            if let Some(message_requests) = self.frame_stage_requests.get(frame_stage.index()) {
                println!("  {frame_stage:?}");

                for message_request in message_requests {
                    if let Some(_event) = self.event_messages.get(&message_request.event_id) {
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
    use crate::event_bus::{Event, EventPayload, FrameStage};

    fn test_event_bus(
        events: &[(EventId, FrameStage)],
        subscribes: &[(EventId, Option<EntityId>)],
        sends: &[(EventId, Option<EntityId>, Option<EventPayload>)],
        expected: &[(FrameStage, TunnelId, Option<EventPayload>)],
    ) {
        let mut event_bus = EventBus::new();
        let event_id_offset = Event::EngineEventsCount.index();

        events.iter().for_each(|(event_id, frame_stage)| {
            let event_id = *event_id + event_id_offset;
            let event_name = format!("TestEvent{event_id}");
            event_bus.register(event_id, &event_name, *frame_stage, true);
        });

        let tunnel_ids: Vec<_> = subscribes
            .iter()
            .map(|(event_id, entity_id)| {
                event_bus.subscribe(*event_id + event_id_offset, *entity_id)
            })
            .collect();

        sends.iter().for_each(|(event_id, entity_id, payload)| {
            event_bus.send(*event_id + event_id_offset, *entity_id, payload.as_ref())
        });

        event_bus.start_event_iteration();

        expected
            .iter()
            .enumerate()
            .for_each(|(i, (frame_stage, tunnel_id, payload))| {
                let event = event_bus
                    .next_event()
                    .unwrap_or_else(|| panic!("Event {i} was not sent"));

                assert!(
                    tunnel_ids.contains(tunnel_id),
                    "[{i}] Unexpected tunnel id: {tunnel_id}"
                );
                assert_eq!(event.frame_stage(), *frame_stage, "[{i}] Frame stage");
                assert_eq!(event.tunnel_id(), *tunnel_id, "[{i}] Tunnel id");
                assert_eq!(event.payload(), payload.as_ref(), "[{i}] Payload");
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
