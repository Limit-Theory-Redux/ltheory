use crate::physics::{PhysicsWorld, PhysicsWorldHandle};
use rapier3d::prelude as rp;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

// struct PhysicsWorldRef<'a> {
//     rc: Rc<RefCell<PhysicsWorld>>,
//     world_ref: Option<Ref<'a, PhysicsWorld>>,
// }

// impl PhysicsWorldRef<'_> {
//     pub fn upgrade<'a>(world: &'a PhysicsWorldHandle) -> PhysicsWorldRef<'a> {
//         let mut r = PhysicsWorldRef {
//             rc: world.upgrade(),
//             world_ref: None,
//         };
//         r.world_ref = Some(r.rc.borrow());
//         r
//     }
// }

pub enum RefOrBorrow<'a, T> {
    Ref(Ref<'a, T>),
    Borrow(&'a T),
}

impl<T> Deref for RefOrBorrow<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            RefOrBorrow::Ref(r) => &**r,
            RefOrBorrow::Borrow(b) => *b,
        }
    }
}

pub enum RefMutOrBorrow<'a, T> {
    Ref(RefMut<'a, T>),
    Borrow(&'a mut T),
}

impl<T> Deref for RefMutOrBorrow<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            RefMutOrBorrow::Ref(r) => &**r,
            RefMutOrBorrow::Borrow(b) => *b,
        }
    }
}

impl<T> DerefMut for RefMutOrBorrow<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match self {
            RefMutOrBorrow::Ref(r) => &mut **r,
            RefMutOrBorrow::Borrow(b) => *b,
        }
    }
}

// A wrapper over either a rigid body or a rigid body handle.
pub(crate) enum RigidBodyWrapper {
    Removed(rp::RigidBody),
    Added(rp::RigidBodyHandle),
}

impl RigidBodyWrapper {
    pub(crate) fn get<'a>(
        &'a self,
        world: &'a Rc<RefCell<PhysicsWorld>>,
    ) -> RefOrBorrow<'a, rp::RigidBody> {
        match self {
            RigidBodyWrapper::Removed(body) => RefOrBorrow::Borrow(body),
            RigidBodyWrapper::Added(handle) => {
                RefOrBorrow::Ref(Ref::map(world.borrow(), |w| w.get(*handle)))
            }
        }
    }

    pub(crate) fn get_mut<'a>(
        &'a mut self,
        world: &'a Rc<RefCell<PhysicsWorld>>,
    ) -> RefMutOrBorrow<'a, rp::RigidBody> {
        match self {
            RigidBodyWrapper::Removed(body) => RefMutOrBorrow::Borrow(body),
            RigidBodyWrapper::Added(handle) => {
                RefMutOrBorrow::Ref(RefMut::map(world.borrow_mut(), |w| w.get_mut(*handle)))
            }
        }
    }
}

/// A wrapper over two states:
/// - A rapier object that does not belong to a world
/// - A rapier object handle that has been added to a world
///
/// This wrapper allows us to access the underlying rapier type regardless of
/// whether it's been added to the world or not.
pub(crate) enum ColliderWrapper {
    Removed(rp::Collider),
    Added(rp::ColliderHandle, Rc<RefCell<PhysicsWorld>>),
}

impl ColliderWrapper {
    pub(crate) fn as_ref<'a>(&'a self) -> RefOrBorrow<'a, rp::Collider> {
        match self {
            ColliderWrapper::Removed(body) => RefOrBorrow::Borrow(body),
            ColliderWrapper::Added(handle, world) => {
                RefOrBorrow::Ref(Ref::map(world.borrow(), |w| w.get(*handle)))
            }
        }
    }

    pub(crate) fn as_mut<'a>(&'a mut self) -> RefMutOrBorrow<'a, rp::Collider> {
        match self {
            ColliderWrapper::Removed(body) => RefMutOrBorrow::Borrow(body),
            ColliderWrapper::Added(handle, world) => {
                RefMutOrBorrow::Ref(RefMut::map(world.borrow_mut(), |w| w.get_mut(*handle)))
            }
        }
    }

    pub(crate) fn replace(&mut self) -> Self {
        std::mem::replace(
            self,
            ColliderWrapper::Added(
                rp::ColliderHandle::invalid(),
                Rc::new(RefCell::new(PhysicsWorld {
                    island_manager: rp::IslandManager::new(),
                    rigid_bodies: rp::RigidBodySet::new(),
                    colliders: rp::ColliderSet::new(),
                })),
            ),
        )
    }

    pub(crate) fn removed_as_ref(&self) -> Option<&rp::Collider> {
        match self {
            ColliderWrapper::Removed(collider) => Some(collider),
            _ => None,
        }
    }

    pub(crate) fn added_as_ref(&self) -> Option<(&rp::ColliderHandle, &Rc<RefCell<PhysicsWorld>>)> {
        match self {
            ColliderWrapper::Added(handle, world) => Some((handle, world)),
            _ => None,
        }
    }

    pub(crate) fn set_added<F>(&mut self, f: F)
    where
        F: FnOnce(rp::Collider) -> (rp::ColliderHandle, Rc<RefCell<PhysicsWorld>>),
    {
        *self = match self.replace() {
            ColliderWrapper::Removed(collider) => {
                let (handle, world) = f(collider);
                ColliderWrapper::Added(handle, world)
            }
            ColliderWrapper::Added(handle, world) => ColliderWrapper::Added(handle, world),
        }
    }

    pub(crate) fn set_removed<F>(&mut self, f: F)
    where
        F: FnOnce(rp::ColliderHandle, Rc<RefCell<PhysicsWorld>>) -> rp::Collider,
    {
        *self = match self.replace() {
            ColliderWrapper::Removed(collider) => ColliderWrapper::Removed(collider),
            ColliderWrapper::Added(handle, world) => ColliderWrapper::Removed(f(handle, world)),
        }
    }

    pub(crate) fn get_handle(&self) -> Option<rp::ColliderHandle> {
        if let ColliderWrapper::Added(handle, _) = self {
            Some(*handle)
        } else {
            None
        }
    }

    pub(crate) fn is_added(&self) -> bool {
        match self {
            ColliderWrapper::Removed(..) => false,
            ColliderWrapper::Added(..) => true,
        }
    }

    pub(crate) fn is_removed(&self) -> bool {
        match self {
            ColliderWrapper::Removed(..) => true,
            ColliderWrapper::Added(..) => false,
        }
    }
}
