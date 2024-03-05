use std::{cell::RefCell, rc::Rc};

use super::{SoundGroup, SoundInstance};

pub struct SoundGroupManager {
    groups: std::collections::HashMap<SoundGroup, Vec<Rc<RefCell<SoundInstance>>>>,
}

impl SoundGroupManager {
    pub fn new() -> Self {
        Self {
            groups: std::collections::HashMap::new(),
        }
    }

    pub fn add_sound(
        &mut self,
        sound_group: SoundGroup,
        sound_instance: Rc<RefCell<SoundInstance>>,
    ) {
        let group = self.groups.entry(sound_group).or_insert_with(Vec::new);
        if group.len() >= sound_group.max_playing_sounds() {
            SoundGroupManager::remove_lowest_volume_sound(group)
        }
        group.push(sound_instance);
    }

    pub fn remove_lowest_volume_sound(group: &mut Vec<Rc<RefCell<SoundInstance>>>) {
        let index_to_remove = group
            .iter()
            .enumerate()
            .min_by_key(|(_, sound_instance)| {
                let sound_instance = sound_instance.borrow();
                (sound_instance.volume * 100.0) as i32
            })
            .map(|(index, _)| index);

        if let Some(index) = index_to_remove {
            if let Some(sound_instance) = group.get(index) {
                if let Some(handle) = &sound_instance.borrow().handle {
                    handle
                        .borrow_mut()
                        .stop(kira::tween::Tween {
                            start_time: kira::StartTime::Immediate,
                            duration: std::time::Duration::from_millis(0),
                            easing: kira::tween::Easing::Linear,
                        })
                        .expect("Failed to stop sound");
                }
            }
            group.remove(index);
        }
    }
}
